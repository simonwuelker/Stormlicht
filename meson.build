project(
  'stormlicht', 
  'rust',
  version: '0.1.0',
  meson_version: '>=1.3.0',
  default_options: [
    'wrap_mode=forcefallback'
  ]
)

gnome = import('gnome')
rust = import('rust')

profile = get_option('profile')

# workspace crates
subdir('stormlicht')

# Setup pre-commit hook for ensuring coding style is always consistent
message('Setting up git pre-commit hook..')
run_command(
  'cp',
  '-f',
  'hooks/pre-commit.hook',
  '.git/hooks/pre-commit',
  check: false,
)
