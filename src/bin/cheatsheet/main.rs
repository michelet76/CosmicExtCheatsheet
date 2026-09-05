// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod cli;

use clap::Parser;
use cosmic::app::Settings;
use cosmic::iced::Size;
use cosmic_ext_cheatsheet::{config::CheatsheetConfig, i18n, registration};

use cli::{Args, Cmd};

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("warn,cosmic_ext_cheatsheet=info")
            }),
        )
        .with_writer(std::io::stderr)
        .init();

    i18n::init_from_desktop();

    let args = Args::parse();

    match args.cmd {
        Some(Cmd::Register { force }) => {
            headless(|ctx| {
                let (mut config, app_ctx) = CheatsheetConfig::load();
                let binding = config.binding.clone();
                if let Err(registration::Error::Conflict(action)) =
                    registration::register_checked(ctx, config.registered.as_ref(), &binding, force)
                {
                    let label =
                        cosmic_ext_cheatsheet::shortcuts::localize::action_label(&action, None);
                    eprintln!(
                        "{} is already bound to \"{label}\"; use --force to replace it",
                        cosmic_ext_cheatsheet::shortcuts::keys::display(&binding)
                    );
                    std::process::exit(2);
                }
                config.registered = Some(binding.clone());
                config.auto_register = true;
                save_app_config(&config, app_ctx.as_ref())?;
                println!(
                    "registered {} -> {}",
                    cosmic_ext_cheatsheet::shortcuts::keys::display(&binding),
                    registration::spawn_command()
                );
                Ok(())
            });
            Ok(())
        }
        Some(Cmd::Unregister) => {
            headless(|ctx| {
                let (mut config, app_ctx) = CheatsheetConfig::load();
                registration::unregister(ctx, config.registered.as_ref())?;
                config.registered = None;
                // An explicit removal must not be undone by the next launch.
                config.auto_register = false;
                save_app_config(&config, app_ctx.as_ref())?;
                println!("unregistered");
                Ok(())
            });
            Ok(())
        }
        Some(Cmd::Settings) => cosmic::app::run_single_instance::<app::App>(
            Settings::default().size(Size::new(640.0, 720.0)),
            args,
        ),
        _ => {
            let started = std::time::Instant::now();
            let result = cosmic::app::run_single_instance::<app::App>(
                Settings::default()
                    .no_main_window(true)
                    .exit_on_close(false)
                    .transparent(true),
                args,
            );
            tracing::debug!("run_single_instance returned after {:?}", started.elapsed());
            result
        }
    }
}

fn headless(f: impl FnOnce(&cosmic::cosmic_config::Config) -> Result<(), registration::Error>) {
    match registration::context() {
        Ok(ctx) => {
            if let Err(why) = f(&ctx) {
                eprintln!("error: {why}");
                std::process::exit(1);
            }
        }
        Err(why) => {
            eprintln!("error: could not open shortcuts config: {why}");
            std::process::exit(1);
        }
    }
}

fn save_app_config(
    config: &CheatsheetConfig,
    ctx: Option<&cosmic::cosmic_config::Config>,
) -> Result<(), registration::Error> {
    match ctx {
        Some(ctx) => config.save(ctx).map_err(registration::Error::Config),
        None => Err(registration::Error::Config(
            cosmic::cosmic_config::Error::NoConfigDirectory,
        )),
    }
}
