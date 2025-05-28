#![allow(clippy::type_complexity)]

use std::{borrow::Cow, fs::File};

use bevy::{
    DefaultPlugins,
    app::{App, AppReload, AppReloader, IsInitialized, Last, Main, Startup, Update},
    color::Color,
    core_pipeline::core_2d::Camera2d,
    ecs::{
        archetype::ArchetypeComponentId,
        children,
        component::{Component, ComponentId, Tick},
        event::EventWriter,
        query::{Access, Changed, FilteredAccessSet, With},
        schedule::Schedules,
        spawn::SpawnRelated,
        system::{Commands, Query, Single, System},
        world::{World, unsafe_world_cell::UnsafeWorldCell},
    },
    ui::{
        AlignItems, BorderColor, Interaction, JustifyContent, Node, UiRect, Val,
        widget::{Button, Text},
    },
    utils::default,
};
use nom::IResult;

#[derive(Debug)]
enum ImagineStage {
    Update,
    Last,
}

#[derive(Debug)]
enum ImagineStatement {
    Print { text: String },
}

#[derive(Debug)]
struct ImagineFunction {
    stage: ImagineStage,
    name: String,
    body: Vec<ImagineStatement>,

    last_run: Tick,
}

impl System for ImagineFunction {
    type In = ();
    type Out = ();

    fn name(&self) -> Cow<'static, str> {
        self.name.to_string().into()
    }

    fn component_access(&self) -> &Access<ComponentId> {
        const EMPTY_ACCESS: &Access<ComponentId> = &Access::new();

        EMPTY_ACCESS
    }

    fn component_access_set(&self) -> &FilteredAccessSet<ComponentId> {
        const EMPTY_ACCESS: &FilteredAccessSet<ComponentId> = &FilteredAccessSet::new();

        EMPTY_ACCESS
    }

    fn archetype_component_access(&self) -> &Access<ArchetypeComponentId> {
        const EMPTY_ACCESS: &Access<ArchetypeComponentId> = &Access::new();

        EMPTY_ACCESS
    }

    fn is_send(&self) -> bool {
        true
    }

    fn is_exclusive(&self) -> bool {
        false
    }

    fn has_deferred(&self) -> bool {
        false
    }

    unsafe fn run_unsafe(
        &mut self,
        _input: bevy::ecs::system::SystemIn<'_, Self>,
        _world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Self::Out {
        self.body.iter().for_each(|statement| match statement {
            ImagineStatement::Print { text } => println!("{text}"),
        })
    }

    fn apply_deferred(&mut self, _world: &mut bevy::ecs::world::World) {
        // do nothing
    }

    fn queue_deferred(&mut self, _world: bevy::ecs::world::DeferredWorld) {
        // do nothing
    }

    unsafe fn validate_param_unsafe(
        &mut self,
        _world: UnsafeWorldCell,
    ) -> Result<(), bevy::ecs::system::SystemParamValidationError> {
        Ok(())
    }

    fn initialize(&mut self, _world: &mut World) {
        // do nothing
    }

    fn update_archetype_component_access(&mut self, _world: UnsafeWorldCell) {
        // do nothing
    }

    fn check_change_tick(&mut self, this_run: Tick) {
        let age = this_run.get().wrapping_sub(self.last_run.get());

        if age > Tick::MAX.get() {
            self.last_run = Tick::new(self.last_run.get().wrapping_sub(Tick::MAX.get()));
        }
    }

    fn get_last_run(&self) -> Tick {
        self.last_run
    }

    fn set_last_run(&mut self, last_run: Tick) {
        self.last_run = last_run
    }
}

#[derive(Debug)]
struct ImagineFile {
    items: Vec<ImagineFunction>,
}

impl ImagineFile {
    fn parse(input: &str) -> Result<ImagineFile, nom::Err<nom::error::Error<&str>>> {
        use nom::{
            Parser,
            branch::alt,
            bytes::complete::{is_not, tag},
            character::complete::{alpha1, alphanumeric1, multispace0, newline},
            combinator::{eof, map, recognize},
            error::ParseError,
            multi::{many0_count, separated_list1},
            sequence::{delimited, pair},
        };

        pub fn ws<'a, O, E: ParseError<&'a str>, F>(
            inner: F,
        ) -> impl Parser<&'a str, Output = O, Error = E>
        where
            F: Parser<&'a str, Output = O, Error = E>,
        {
            delimited(multispace0, inner, multispace0)
        }

        fn identifier(input: &str) -> IResult<&str, &str> {
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_")))),
            ))
            .parse(input)
        }

        fn statement(input: &str) -> IResult<&str, ImagineStatement> {
            let (input, _) = ws(tag("print")).parse(input)?;
            let (input, _) = tag("\"")(input)?;
            let (input, text) = is_not("\"")(input)?;
            let (input, _) = tag("\"")(input)?;

            Ok((
                input,
                ImagineStatement::Print {
                    text: text.to_string(),
                },
            ))
        }

        fn function_stage(input: &str) -> IResult<&str, ImagineStage> {
            let stage_name = alt((
                map(tag("Update"), |_| ImagineStage::Update),
                map(tag("Last"), |_| ImagineStage::Last),
            ));

            ws(delimited(tag("#["), stage_name, tag("]"))).parse(input)
        }

        fn function(input: &str) -> IResult<&str, ImagineFunction> {
            let (input, stage) = function_stage(input)?;

            let (input, _) = tag("fn ")(input)?;
            let (input, name) = identifier(input)?;
            let (input, _) = tag("()")(input)?;
            let (input, _) = ws(tag("{")).parse(input)?;
            let (input, statements) = separated_list1(newline, statement).parse(input)?;
            let (input, _) = ws(tag("}")).parse(input)?;

            Ok((
                input,
                ImagineFunction {
                    stage,
                    name: name.to_string(),
                    body: statements,

                    last_run: Tick::default(),
                },
            ))
        }

        let (input, items) = separated_list1(multispace0, function).parse(input)?;
        let (_, _) = eof(input)?;

        Ok(ImagineFile { items })
    }

    fn apply(self, app: &mut App) {
        self.items
            .into_iter()
            .for_each(|function| match function.stage {
                ImagineStage::Update => {
                    app.add_systems(Update, function);
                }
                ImagineStage::Last => {
                    app.add_systems(Last, function);
                }
            });
    }
}

// This function and the fact we are recreating the app is a hack around the fact that systems can't
// currently be removed from schedules
//
// I think the right API for this is something that _consumes_ the `Schedules` and returns an
// iterator that can be filtered/modified (containing systems and their run conditions etc) and then
// `Schedules` should be reconstructable from that iterator
fn build_app(app: &mut App) {
    let file = std::fs::read_to_string("./mods/main.imagine")
        .expect("main.imagine to exist in mods folder");
    let file = ImagineFile::parse(&file).expect("invalid imagine file");

    file.apply(app);

    app.add_plugins(DefaultPlugins)
        .insert_resource(AppReloader(Box::new(|current_app| {
            let mut new_app = App::new();
            new_app.world_mut().init_resource::<IsInitialized>();

            build_app(&mut new_app);

            let mut updated_schedules = new_app.world_mut().remove_resource::<Schedules>().unwrap();
            if let Some(new_main_schedule) = updated_schedules.get_mut(Main) {
                let mut old_schedules = current_app.world_mut().resource_mut::<Schedules>();
                let old_main_schedule = old_schedules
                    .get_mut(Main)
                    .expect("new world has main but old one doesn't");

                std::mem::swap(new_main_schedule, old_main_schedule);
            }
            current_app.insert_resource(updated_schedules);
        })))
        .add_systems(Startup, setup)
        .add_systems(Update, button_clicked);
}

fn main() {
    let mut app = App::new();
    build_app(&mut app);

    app.run();
}

#[derive(Component)]
struct ButtonText;

fn button_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut text: Single<&mut Text, With<ButtonText>>,
    mut app_reload: EventWriter<AppReload>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            text.0 = "Reloading...".to_string();

            app_reload.write(AppReload);
        } else {
            text.0 = "Reload Mods".to_string();
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                padding: UiRect::axes(Val::Px(50.0), Val::Px(20.0)),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            children![(Text::new("Reload Mods"), ButtonText)]
        )],
    ));
}
