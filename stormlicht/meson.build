subdir('resources')

executable(
    'stormlicht', 
    'src/main.rs',
    'src/chrome/mod.rs',
    'src/chrome/gtk/mod.rs',
    'src/chrome/gtk/run.rs',
    'src/chrome/gtk/window/imp.rs',
    'src/chrome/gtk/window/mod.rs',
    dependencies: [
        dependency('env_logger-rs')
    ]
)