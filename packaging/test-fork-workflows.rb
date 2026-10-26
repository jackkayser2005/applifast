require 'yaml'

ROOT = File.expand_path('..', __dir__)
UPSTREAM = "github.repository == 'crmne/spotifast'"
GUARDED_JOBS = {
  'docs.yml' => %w[build deploy],
  'release.yml' => %w[build macos flatpak publish packaging],
  'flatpak-from-release.yml' => %w[flatpak]
}.freeze

def upstream_only?(expression)
  body = expression.to_s.strip.sub(/\A\$\{\{\s*/, '').sub(/\s*\}\}\z/, '').strip
  # These guards are conjunctions. A top-level OR could bypass the boundary.
  body == UPSTREAM || (body.start_with?("#{UPSTREAM} && ") && !body.include?('||'))
end

def problems(workflows)
  errors = []
  GUARDED_JOBS.each do |file, jobs|
    jobs.each do |name|
      job = workflows.fetch(file).fetch('jobs').fetch(name)
      errors << "#{file}/#{name} lacks an upstream publishing guard" unless upstream_only?(job['if'])
    end
  end

  packaging = workflows.fetch('packaging.yml').fetch('jobs').fetch('packaging')
  publish = packaging.fetch('with').fetch('publish')
  unless upstream_only?(publish) && publish.include?('inputs.publish == true') && publish.include?("inputs.version != ''")
    errors << 'packaging.yml permits fork publication or loses explicit publish/version checks'
  end
  errors << 'Fork packaging fixtures must remain enabled' if packaging.key?('if')
  publishing_secrets = %w[AUR_SSH_KEY AUR_KNOWN_HOSTS HOMEBREW_TAP_GITHUB_TOKEN HOMEBREW_TAP_SSH_KEY]
  forwarded = packaging.fetch('secrets')
  unless forwarded.is_a?(Hash) && forwarded.keys.sort == publishing_secrets.sort
    errors << 'External native packaging must receive only its declared publishing secrets'
  end

  ci_workflow = workflows.fetch('ci.yml')
  ci_events = ci_workflow.fetch('on') { ci_workflow.fetch(true) }
  unless ci_events.key?('pull_request') && ci_events.fetch('push').fetch('branches').include?('main')
    errors << 'Fork CI must retain pull-request and main-push triggers'
  end
  ci = ci_workflow.fetch('jobs')
  %w[quality test docs nix].each do |name|
    errors << "CI #{name} must remain available to forks" unless ci.key?(name) && !ci.fetch(name).key?('if')
  end
  platforms = ci.fetch('test').fetch('strategy').fetch('matrix').fetch('os')
  unless %w[ubuntu-latest macos-latest windows-latest windows-11-arm].all? { |platform| platforms.include?(platform) }
    errors << 'Fork publishing guards must not remove platform coverage'
  end

  preview = workflows.fetch('windows-preview.yml')
  events = preview.fetch('on') { preview.fetch(true) }
  unless events.keys == ['workflow_dispatch'] && preview.fetch('permissions').fetch('contents') == 'read'
    errors << 'Windows preview must retain manual dispatch and read-only contents permission'
  end
  if preview.fetch('jobs').fetch('preview').key?('if')
    errors << 'Windows preview must remain available to the fork'
  end
  errors
end

names = GUARDED_JOBS.keys + %w[packaging.yml ci.yml windows-preview.yml]
workflows = names.to_h do |name|
  [name, YAML.safe_load_file(File.join(ROOT, '.github', 'workflows', name), aliases: true)]
end
errors = problems(workflows)
raise errors.join("\n") unless errors.empty?

# Reject both an accidentally removed guard and a disjunctive bypass at every
# publishing entry point, while checking the ordinary CI/preview paths above.
GUARDED_JOBS.each do |file, jobs|
  jobs.each do |name|
    [nil, "#{UPSTREAM} || true"].each do |bad_guard|
      changed = Marshal.load(Marshal.dump(workflows))
      changed.fetch(file).fetch('jobs').fetch(name)['if'] = bad_guard
      raise "Missing regression coverage for #{file}/#{name}" if problems(changed).empty?
    end
  end
end
["${{ inputs.publish == true && inputs.version != '' }}", "${{ #{UPSTREAM} || true }}"].each do |bad_guard|
  changed = Marshal.load(Marshal.dump(workflows))
  changed.fetch('packaging.yml').fetch('jobs').fetch('packaging').fetch('with')['publish'] = bad_guard
  raise 'Missing reusable publisher regression coverage' if problems(changed).empty?
end

changed = Marshal.load(Marshal.dump(workflows))
changed.fetch('packaging.yml').fetch('jobs').fetch('packaging')['secrets'] = 'inherit'
raise 'Missing reusable secret-scope regression coverage' if problems(changed).empty?

puts 'Fork publishing guards, negative regressions and retained CI/Windows preview paths pass.'
