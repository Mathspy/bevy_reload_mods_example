# Bevy reloading mods example

This is an example powered by
[`AppReload`](https://github.com/Mathspy/bevy/tree/app-reload) feature that
behaves as if there's a mod/script loaded in as part of the schedule without any
special mutable access wrappers and then demonstrates the mod being reloaded

Besides relying on the `AppReload` PROPOSED feature (not merged in yet), it does
a hack where it recreates the full app to get the old schedules. There's work
being done in this area around allowing
[`Schedules`](https://docs.rs/bevy_ecs/0.16.0/bevy_ecs/schedule/struct.Schedules.html)
to be updated after the fact. One interesting aspect of this work that makes it
a lot easier to update schedules with the slightest upstream changes, is that it
is accepts the idea that reloading the app can be a slow process and that's
okay.

This is one of the biggest hurdles around updating schedules today in Bevy,
updating schedules requires rebuilding the execution graph which isn't a fast
process and should probably not be done _during_ regular execution. `AppReload`
with `AppReloader` allows each end user to opt into expensive operations only if
they think they are necessary, and asks the end user to do such expensive things
sparingly.

For example an API that looks like the one below should be enough for this
implementation to work without any hacks

```rs
impl Schedules {
  fn into_systems(self) -> impl Iterator<(Box<dyn ScheduleLabel>, Box<dyn System>)>;

  fn from_systems(systems: impl Iterator<(Box<dyn ScheduleLabel>, Box<dyn System>)>) -> Self;
}
```

## How to run/test

This example doesn't need any special support to run:

> [!WARNING]\
> This example spams logs by default, use a new terminal window if you don't
> want your terminal to be buried

```rs
cargo run
```

To test, update the [`main.imagine`](./mods/main.imagine) file and then click
the reload button. I suggest removing the `print_yay` system, reloading then
adding it back and reloading again.
