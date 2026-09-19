# AeoScript

AeoEngine's gameplay scripting language.

**File extension:** `.aeo`

**Current status:** Early design

---

## Why are we making this?

AeoEngine needs a scripting layer.

Rust is the engine language. It is great for building the engine itself, but it is not what we want people using every time they want a door to open, an enemy to chase the player, or a light to turn on.

The goal of AeoScript is simple:

> Make game behavior easy to write without exposing the engine's internals.

Someone should be able to create an entity, attach a script, open Monaco, write a few lines, save it, and see the thing work.

That is the point.

---

# 1. What AeoScript should feel like

AeoScript is its own language.

It should be:

* Easy to read
* Easy to write
* Small enough to understand
* Structured enough to grow
* Closely integrated with AeoEngine
* Explicit about the boundary between game code and engine code

Types should be available when they are useful.

Inference should be available when the type is obvious.

The language should stay small for a long time.

AeoScript should not become a second Rust.

---

# 2. What a script looks like

A script can be attached to an entity.

```aeoscript
entity Door {

    open: bool = false

    fn update(dt: number) {
    }

    fn interact(player: Entity) {
        open = !open
    }
}
```

The important thing is that this should be readable without knowing how AeoEngine is implemented internally.

---

# 3. Script files

Scripts use:

```text
.aeo
```

Examples:

```text
Player.aeo
Door.aeo
Enemy.aeo
Guard.aeo
MainMenu.aeo
```

Scripts are normal project files.

Nothing about the scripting system should require a hidden database or proprietary editor format.

---

# 4. The basic language

The first version should contain the things we actually need.

Initial types:

```text
number
bool
string
Entity
Cell
Basket
vec2
vec3
vec4
```

Optional values:

```text
Entity?
Cell?
number?
string?
```

Examples:

```aeoscript
health = 100
speed = 6.0
alive = true
name = "Guard"
target = nil
```

Types can be written explicitly:

```aeoscript
health: number = 100
speed: number = 6.0
alive: bool = true
```

Type inference is allowed where it is obvious.

---

# 5. Variables

Variables are mutable by default.

```aeoscript
health = 100

health -= 10
```

There is no reason to make people write `mut` everywhere just to change a value.

Constants are available when needed:

```aeoscript
const MAX_HEALTH = 100
```

Constants cannot be reassigned.

---

# 6. Functions

Functions use `fn`.

```aeoscript
fn heal(amount: number) {
    health += amount
}
```

Return values:

```aeoscript
fn is_alive(): bool {
    return health > 0
}
```

A function does not need a special syntax just because it does not return a value.

---

# 7. Conditions

Normal game code should look like this:

```aeoscript
if health <= 0 {
    die()
}
```

With `else`:

```aeoscript
if health <= 0 {
    die()
} else {
    keep_fighting()
}
```

And:

```aeoscript
if health < 25 {
    retreat()
} else if health < 50 {
    defend()
} else {
    attack()
}
```

Boolean operators:

```text
&&
||
!
```

Example:

```aeoscript
if alive && health > 0 {
    attack()
}
```

---

# 8. Loops

We need normal loops, but scripts should not be able to accidentally lock the engine indefinitely.

Basic `while`:

```aeoscript
while timer > 0 {
    timer -= 1
}
```

Basic `for`:

```aeoscript
for enemy in enemies {
    enemy:highlight()
}
```

Collections use the AeoScript Basket type.

The runtime may impose execution limits where appropriate.

---

# 9. Comments

Single line:

```aeoscript
// This opens the door
```

Multi-line:

```aeoscript
/*
    Temporary test code.
*/
```

Nothing fancy here.

---

# 10. Entities

An `Entity` is a live runtime object.

Example:

```aeoscript
entity Enemy {

    health: number = 100
    speed: number = 3.5

    fn update(dt: number) {
        // enemy logic
    }
}
```

The engine owns the actual runtime entity.

The script does not.

A script receives an engine-managed reference and uses the API the engine exposes.

Entity references can be passed to functions, stored in variables, placed in Baskets, and received as event arguments.

---

# 11. Cells

A `Cell` represents authored world content.

Examples include:

```text
Block
Light
SpawnPoint
future authored objects
```

Cells are safe script-facing references to authored content.

They are not live runtime Entities.

A Cell is read-only from the scripting side except for operations explicitly exposed by the engine.

A Cell may provide access to a live runtime object:

```aeoscript
object = cell:getObject()
```

This returns an `Entity` when one exists, otherwise `nil`.

---

# 12. Baskets

A `Basket` is AeoScript's collection type.

Baskets are 0-based.

Example:

```aeoscript
items = [1, 2, 3, 4]

value = items[0]
count = items.len()
```

Baskets may contain supported AeoScript values, including:

```text
Cells
Entities
Baskets
numbers
strings
booleans
nil
```

Baskets are used for things such as:

```aeoscript
children = object:get_children()

for child in children {
    print(child.name)
}
```

The language does not use Lua-style `#` length syntax.

---

# 13. Nil

AeoScript uses:

```aeoscript
nil
```

to represent the absence of a value.

Example:

```aeoscript
target: Entity? = nil
```

For example:

```aeoscript
object = cell:getObject()

if object == nil {
    return
}
```

There is no `null` literal.

---

# 14. Object Access

AeoScript uses a consistent object model.

Properties use `.`:

```aeoscript
player.name
player.position
light.enabled
```

Methods use `:`:

```aeoscript
player:destroy()
character:jump()
light:set_enabled(false)
```

This applies to supported engine objects as well as normal AeoScript objects where appropriate.

The engine decides which properties and methods an object exposes.

---

# 15. Parent and Children

Objects may expose their relationships through the generic object model.

Example:

```aeoscript
parent = object:get_parent()
children = object:get_children()
```

`get_children()` returns usable object references in a Basket.

`get_parent()` returns the appropriate parent object when one exists.

An unattached script does not have a parent.

---

# 16. Discovery

Scripts need to be able to find objects without requiring a special function for every engine object type.

For example:

```aeoscript
lights = getAllCellsOfClass("Light")
```

This returns a Basket of Cell references.

Generic discovery may also look like:

```aeoscript
objects = find("Door")
```

The exact query behavior is an engine/API decision.

The important rule is that new engine objects should normally use the existing object model instead of adding:

```text
get_light()
get_player()
get_npc()
get_door()
get_vehicle()
```

and so on.

---

# 17. Events

Events are a first-class part of AeoScript.

Example:

```aeoscript
on PlayerSpawned(player) {
    print("Player spawned")
}
```

The engine owns event dispatch.

AeoScript receives the event and its arguments.

Events use a generic syntax so future engine events do not require new language syntax.

Examples:

```aeoscript
on DoorOpened(door) {
}

on EntityDestroyed(entity) {
}

on ButtonPressed(button) {
}
```

The first concrete engine event is `PlayerSpawned`.

---

# 18. Lifecycle

Scripts need standard lifecycle entry points.

Initial set:

```aeoscript
fn on_spawn() {
}

fn on_ready() {
}

fn update(dt: number) {
}

fn on_destroy() {
}
```

Additional gameplay behavior should use the event system where appropriate.

For example:

```aeoscript
on Interact(player) {
    open_door()
}
```

The engine controls when lifecycle functions and events are invoked.

---

# 19. Update

The engine provides `dt`.

```aeoscript
fn update(dt: number) {
    timer -= dt
}
```

The script should not create its own game loop.

Avoid:

```aeoscript
while true {
    do_something()
}
```

Prefer:

```aeoscript
fn update(dt: number) {
    do_something()
}
```

The engine controls the clock and execution lifecycle.

---

# 20. Engine APIs

AeoScript becomes useful through AeoEngine's APIs.

The initial API areas include:

```text
entity
transform
character
physics
input
audio
world
camera
debug
```

This is a starting point, not a promise that every API exists immediately.

The APIs must match the actual engine.

We should not invent a giant fake API just to fill out this document.

---

# 21. Transform

Possible properties:

```aeoscript
transform.position
transform.rotation
transform.scale
```

Possible methods:

```aeoscript
transform:move(...)
transform:move_forward(...)
transform:rotate(...)
transform:look_at(...)
```

Exact behavior should follow AeoEngine's transform implementation.

---

# 22. Character

Character scripts should be able to give the character commands without knowing how the character controller works internally.

Example:

```aeoscript
character:move(direction)
character:jump()
character:look(direction)
```

Possibly:

```aeoscript
character:is_grounded()
```

The character controller stays in Rust.

AeoScript provides intent.

---

# 23. Physics

Gameplay scripts need useful physics operations.

For example:

```aeoscript
physics:apply_force(force)
physics:apply_impulse(impulse)
velocity = physics.velocity
```

Scripts should not manipulate the collision solver directly.

More advanced physics functionality can be added when there is an actual gameplay need.

---

# 24. Input

Input should be simple.

```aeoscript
if input:is_down("W") {
    ...
}
```

And:

```aeoscript
if input:was_pressed("Space") {
    character:jump()
}
```

Eventually named actions should be supported:

```aeoscript
input:is_down("move_forward")
input:was_pressed("jump")
```

Raw keys are useful for development and testing.

Named actions are more useful for actual game logic.

---

# 25. Audio

Something simple:

```aeoscript
audio:play("door_open")
```

Possibly:

```aeoscript
audio:stop("alarm")
```

The script should not need to know what audio backend AeoEngine is using.

---

# 26. World

Scripts need controlled access to world operations.

For example:

```aeoscript
world:find("Door")
```

Eventually:

```aeoscript
world:spawn("Enemy", position)
```

The exact world API depends on the systems AeoEngine actually supports.

The engine should expose useful operations rather than exposing every internal world structure.

---

# 27. Camera

Possible operations:

```aeoscript
camera:look_at(position)
camera:set_mode("follow")
```

The final camera API depends on how gameplay camera control is exposed by AeoEngine.

---

# 28. Debug

Development scripts should be able to write useful information.

For example:

```aeoscript
debug:log("Enemy spawned")
debug:warn("No target")
```

Possible visual debugging:

```aeoscript
debug:draw_line(start, finish)
debug:draw_point(position)
```

These are development tools, not gameplay dependencies.

---

# 29. Entity References

A script can hold an Entity reference:

```aeoscript
target: Entity? = nil
```

The engine owns the referenced object.

If the entity is destroyed, the script must not be left with a dangling native pointer.

The VM should detect an invalid reference and produce a clean result according to the API being used.

It must never crash the engine.

---

# 30. Script State

A script can keep its own state.

```aeoscript
entity Door {

    open: bool = false
    timer: number = 0

    fn update(dt: number) {
        if open {
            timer += dt
        }
    }
}
```

Each script instance has its own state.

Two entities using the same script should not accidentally share mutable fields.

---

# 31. Collections

Collections are important because game scripts will use them constantly.

Baskets are the initial collection type.

Example:

```aeoscript
items = [1, 2, 3, 4]

for item in items {
    print(item)
}
```

Access:

```aeoscript
value = items[0]
```

Size:

```aeoscript
count = items.len()
```

More advanced collection types can be added later when there is a real need.

---

# 32. Maps

A map/dictionary type may be useful later.

Possible syntax:

```aeoscript
stats = {
    health: 100,
    armor: 25
}
```

This is not a first-day requirement.

Do not add it until the basic collection/runtime model is stable.

---

# 33. Modules

Multiple scripts will eventually need to share code.

Possible direction:

```aeoscript
import Shared.Math
```

This should be implemented after the basic language and runtime are working.

There is no reason to build a large module system before the core scripting workflow is useful.

---

# 34. Errors

Errors need to be useful.

Example:

```text
AeoScript Error

Door.aeo:18:9

Unknown property:
    transform.velcity

Did you mean:
    transform.velocity
```

Runtime example:

```text
AeoScript Runtime Error

Enemy.aeo:41

Attempted to use a destroyed entity.
```

Errors should point back to the source file and location whenever possible.

A scripting error should not take down the engine.

---

# 35. Sandboxing

AeoScript is a gameplay language, not a way to execute arbitrary code on the user's machine.

Scripts should not have direct access to:

```text
filesystem
shell commands
process creation
native DLL loading
raw pointers
OpenGL
OS APIs
arbitrary network sockets
```

A script gets the AeoEngine APIs that are explicitly exposed to it.

That boundary needs to stay strong.

---

# 36. Runtime

The intended execution model is:

```text
AeoScript
    |
    v
Lexer
    |
    v
Parser
    |
    v
AST
    |
    v
Compiler / Runtime
    |
    v
AeoScript VM
    |
    v
AeoEngine
```

The exact compiler and bytecode architecture can continue to evolve.

The important boundary is that AeoScript runs inside a controlled runtime rather than becoming native engine code.

---

# 37. Native API Boundary

AeoScript can only call native engine functionality that has been explicitly exposed.

Conceptually:

```text
AeoScript
    |
    v
VM
    |
    v
Registered native API
    |
    v
AeoEngine
```

There is no generic "call Rust" operation.

Native APIs should have known:

* Parameters
* Return values
* Type information
* Runtime behavior

---

# 38. No Rust Code Generation

AeoScript should not work by generating Rust source code and compiling it.

The intended model is:

```text
AeoScript → Runtime
```

not:

```text
AeoScript → generated Rust → rustc → native code
```

This keeps execution, diagnostics, and the engine boundary under our control.

---

# 39. Monaco

AeoScript gets a proper editor through Monaco.

The editor should eventually provide:

```text
syntax highlighting
diagnostics
autocomplete
hover
go-to-definition
formatting
search
script navigation
```

The editor should understand the actual AeoScript language.

It should not slowly become a second parser.

---

# 40. Play Mode Workflow

The intended workflow is:

```text
Select Entity
    |
    v
Add Script
    |
    v
Create .aeo
    |
    v
Open Monaco
    |
    v
Write Script
    |
    v
Save
    |
    v
Compile / Validate
    |
    v
Play
```

Eventually:

```text
edit
save
reload
```

should be fast enough that scripting feels interactive.

---

# 41. Project Layout

Something like:

```text
AeoEngineProject/
|
├── .assets/
|
├── docs/
│   └── language/
│       ├── AeoScript_MASTER.md
│       ├── AeoScript_GRAMMAR.md
│       └── AeoScript_API.md
|
├── scripts/
│   ├── Player.aeo
│   ├── Door.aeo
│   └── Enemy.aeo
|
├── scenes/
|
└── project.aeo
```

The exact project format can change.

The important part is that scripts remain visible, normal files.

---

# 42. Development Diagnostics

Development builds should eventually provide useful scripting information such as:

```text
script compile time
script execution time
runtime errors
instruction count
reload count
event count
```

This will help identify scripts that are doing excessive work.

---

# 43. Multiplayer

Multiplayer needs to distinguish between:

```text
local state
authority state
replicated state
remote events
```

A script running on one machine should not automatically imply that its state is authoritative everywhere.

Networking should be designed into the runtime boundary, but it should not dominate the first usable scripting release.

---

# 44. Saving and Loading

Script state may eventually become part of save data.

For example:

```aeoscript
entity Door {

    open: bool = false
    locked: bool = true
}
```

Not every runtime value should be serializable.

Temporary values, active runtime references, and engine handles need separate treatment.

---

# 45. Hot Reload

Hot reload is an important eventual goal.

A developer should eventually be able to:

```text
Play
    |
    v
Edit Door.aeo
    |
    v
Save
    |
    v
Compile
    |
    v
Reload
```

without restarting the entire game.

Whether existing script state survives a reload is a separate decision.

---

# 46. Performance

AeoScript does not need to be as fast as Rust.

It needs to be fast enough that normal gameplay scripting is not a problem.

The engine remains responsible for expensive systems such as:

```text
rendering
physics
animation
world processing
resource loading
```

Scripts should primarily coordinate those systems.

For example:

```aeoscript
if player_near {
    door:open()
}
```

is exactly the kind of work AeoScript should do.

---

# 47. Compiler and Runtime Diagnostics

Syntax errors:

```text
Door.aeo:12

Expected ')'
```

Type errors:

```text
Player.aeo:24

Expected number.
Got string.
```

Unknown names:

```text
Enemy.aeo:31

Unknown function:
    attackk()

Did you mean:
    attack()
```

The compiler/runtime should give enough information to fix the problem without requiring the developer to inspect engine source code.

---

# 48. Language Server

Eventually Monaco should communicate with an AeoScript language service.

Something like:

```text
Monaco
   |
   v
AeoScript Language Service
   |
   +-- parser
   +-- symbols
   +-- types
   +-- diagnostics
   +-- completions
   +-- documentation
```

The compiler and language service should share as much language infrastructure as practical.

One language should have one parser.

---

# 49. Script Attachments

AeoEngine needs to know which script belongs to which entity.

Conceptually:

```text
Entity
├── Transform
├── PhysicsBody
├── Character
└── Script
    └── Player.aeo
```

The exact internal representation is still an engine decision.

The important part is that the relationship between an entity and its script is explicit.

Unattached scripts should also be possible for server/gameplay-style logic that discovers objects and responds to events.

---

# 50. No Giant Standard Library

The first version does not need hundreds of built-in functions.

AeoScript is useful because AeoEngine exposes useful functionality.

For example:

```aeoscript
character:jump()
audio:play("jump")
```

is more valuable than building a huge standard library before the engine APIs exist.

Keep the language small.

Build the engine API around it.

---

# 51. Debugging

A developer should eventually be able to see something like:

```text
AeoScript

Player.aeo
Line 24
update()

Execution: 0.031 ms
Instructions: 42
```

Runtime errors should point back to source files.

Monaco should eventually be able to stop on a script line and inspect script variables.

That is later work, but the architecture should leave room for it.

---

# 52. Versioning

AeoScript will eventually need language versions.

For example:

```text
AeoEngine 0.7
AeoScript 0.1
```

The language version and engine version do not necessarily need to match.

Old scripts should not silently change meaning after an engine update.

Breaking language changes should be obvious and documented.

---

# 53. First Usable Release

The first usable version should contain the core things needed for real gameplay:

```text
variables
constants
numbers
booleans
strings
nil
optional values
functions
return values
arithmetic
comparisons
logical operators
if / else
while
for
entities
Cells
Baskets
property access
method calls
events
lifecycle functions
basic engine APIs
```

That is enough to start making actual game behavior.

Everything else can come after that.

---

# 54. Things We Are Deliberately Not Deciding Yet

These remain open:

```text
exact bytecode format
VM instruction set
closures
anonymous functions
modules
maps
advanced collections
coroutines
yield
async
networking details
reflection
generics
pattern matching
operator overloading
inheritance
interfaces
```

Some of these may not belong in AeoScript at all.

We do not need to solve everything before the core language works.

---

# 55. Language Philosophy

AeoScript should make this:

```aeoscript
if player_near {
    door:open()
}
```

feel better than exposing a pile of engine internals.

It should make this:

```aeoscript
character:jump()
```

feel better than manually manipulating controller state.

It should make this:

```aeoscript
audio:play("alarm")
```

feel better than exposing the audio implementation.

The language exists to make AeoEngine easier to use.

---

# 56. Example Player Script

```aeoscript
entity Player {

    speed: number = 6.0

    on PlayerSpawned(player) {
        print("Player spawned")
    }

    fn update(dt: number) {

        direction = vec3(0, 0, 0)

        if input:is_down("W") {
            direction.z += 1
        }

        if input:is_down("S") {
            direction.z -= 1
        }

        if input:is_down("A") {
            direction.x -= 1
        }

        if input:is_down("D") {
            direction.x += 1
        }

        if direction:length() > 0 {
            character:move(
                direction:normalized() * speed * dt
            )
        }

        if input:was_pressed("Space") {
            character:jump()
        }
    }
}
```

---

# 57. Example Door

```aeoscript
entity Door {

    open: bool = false

    fn interact(player: Entity) {

        if open {
            close()
        } else {
            open_door()
        }
    }

    fn open_door() {
        open = true
        audio:play("door_open")
    }

    fn close() {
        open = false
        audio:play("door_close")
    }
}
```

---

# 58. Example Enemy

```aeoscript
entity Enemy {

    health: number = 100
    target: Entity? = nil
    speed: number = 3.5

    fn update(dt: number) {

        if target != nil {

            direction = target.position - transform.position

            if direction:length() > 1 {
                character:move(
                    direction:normalized() * speed * dt
                )
            }
        }
    }

    on Damage(amount) {

        health -= amount

        if health <= 0 {
            die()
        }
    }

    fn die() {
        debug:log("Enemy defeated")
        entity:destroy()
    }
}
```

---

# 59. Development Order

The implementation should happen in useful layers.

### First

Build the language core:

```text
lexer
parser
AST
expressions
statements
functions
diagnostics
```

### Then

Build the runtime:

```text
values
variables
functions
execution
limits
events
```

### Then

Connect it to AeoEngine:

```text
Entity
Cell
Basket
Transform
Input
Debug
Character
```

### Then

Build the scripting editor:

```text
Monaco
syntax highlighting
diagnostics
autocomplete
hover
formatting
```

### Then

Expand the engine API:

```text
Physics
Audio
World
Camera
additional gameplay systems
```

This keeps the language useful at every stage instead of building a huge amount of infrastructure before anything actually works.

---

# 60. Current Decisions

These are the decisions established for now:

```text
Name:
    AeoScript

Extension:
    .aeo

Purpose:
    AeoEngine gameplay scripting

Native engine language:
    Rust

Execution:
    Controlled AeoScript runtime / VM

Editor:
    Monaco

Language style:
    Small
    readable
    gameplay-oriented
    typed where useful
    inference where practical

Core object model:
    Cell
    Entity
    Basket

Property syntax:
    object.property

Method syntax:
    object:method(...)

Events:
    on EventName(args) { ... }

Nil value:
    nil

Scripts:
    Attached or unattached

Engine ownership:
    Engine owns runtime objects

Security:
    Explicit native API boundary

Initial priority:
    Small, useful language
```

---

# 61. The Goal

The final experience should be something like this:

```text
Create Entity
      |
      v
Add Script
      |
      v
New Enemy.aeo
      |
      v
Monaco
      |
      v
Write code
      |
      v
Save
      |
      v
Compile
      |
      v
Play
```

And then:

The enemy moves.

The door opens.

The light turns on.

The player jumps.

That is what AeoScript is for.

---

## Status

This document is the current master design for AeoScript.

It is expected to change as the implementation develops.

When the implementation forces a design decision, update this document rather than letting the code and language become two different things.

**AeoScript — early design**
