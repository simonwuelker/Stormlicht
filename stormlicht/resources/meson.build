# The directory where compiled and bundled
# resources will be stored
resource_dir = '.'

# Compile .blp files to .ui 
message('Compiling gtk blueprints')
blueprints = custom_target(
  'blueprints',
  input: files(
    'window.blp'
  ),
  output: resource_dir,
  command: [
    find_program('blueprint-compiler', required: true), 
    'batch-compile', 
    '@OUTPUT@', 
    '@CURRENT_SOURCE_DIR@', 
    '@INPUT@'
  ],
)

message(meson.project_build_root() + '/gresources.bundle')

# Compile the .ui files into GNOME resources
gtk_resources = gnome.compile_resources(
  'resources',
  'gresources.xml',
  gresource_bundle: true,
  source_dir: resource_dir,
  dependencies: blueprints,
  install: true,
  install_dir: meson.project_build_root() + '/gresources.bundle'
)
