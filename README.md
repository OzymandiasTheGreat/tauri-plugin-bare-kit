# Tauri Plugin BareKit

> Write your backend logic once in JavaScript and have it run effortlessly across Android, iOS, macOS, Windows, and Linux. No sidecar needed, no complex configuration, no headaches 🐼

## About Bare and BareKit

### [Bare](https://github.com/holepunchto/bare)

Bare is, in the words of it's creators:

> "Small and modular JavaScript runtime for desktop and mobile" - Holepunch Inc.

You can think of Bare as a mininode, tho it's actually much more than that. Bare was created with the goal of running JavaScript on mobile efficiently, without losing support for desktop platforms. Thus it's highly embeddable and treats every platform as a first class citizen.

Bare was built on V8 and libuv, similar to Node. What's more, Bare has built-in support for N-API modules, in addition to it's own addon system. Thus you can make use of the sizable and growing Bare ecosystem or use the vast majority of modules available on NPM.

### [BareKit](https://github.com/holepunchto/bare-kit)

BareKit is an SDK for Bare for native application development. It makes embedding Bare in your application almost trivial.

> \<BareKit\> [...] provides a web worker-like API to start and manage isolated Bare threads, called worklets1, that expose an IPC abstraction with bindings for various languages.

### This plugin

`tauri-plugin-bare-kit` provides BareKit bindings for rust through FFI and exposes familiar API for running Bare worklets to Tauri frontend.

It also provides autolinking of native addons for JavaScript to avoid headaches and frustrations for users (DevEx matters!), so you can just `tauri plugin add tauri-plugin-bare-kit` and start installing packages from NPM and writing your code. When you build your app, everything will be included and correctly linked.

## Installation

```console
tauri plugin add
```

### Note about iOS

Tauri's iOS build heavily depends on XCode so autolinking is trickier to implement. If you haven't yet generated iOS project at the time you install `tauri-plugin-bare-kit` `xcodebuild` will fail to find BareKit.
Just run `npm install` again after generating iOS project and everything should work.

## Usage

```typescript
import { Worklet } from "tauri-plugin-bare-kit-api"
import b4a from "b4a"
import bundle from "../app.bundle.json"

const worklet = await Worklet.init({ assets, memoryLimit })

await worklet.start("/app.bundle", bundle, args)

const { IPC } = worklet

IPC.on("data", (data) => console.log(b4a.toString(data)))
IPC.write(b4a.from("Hello from Tauri!"))
```

The bundle is automatically generated from your sources and used node modules during build and placed at the root next to package.json.
You should add `app.bundle.json` and `app.bundle.json.d` to your `.gitignore` and ignore initial warnings from your IDE about missing import.

### API

#### `import { Worklet }`

The primary interface for starting, stopping, suspending, and resuming Bare. Because Tauri IPC is asynchronous, you **MUST NOT** instantiate this class directly (e.g. via `new Worklet()`). Instead use the `init(...)` static method.

#### `const worklet = Worklet.init(options)`

Instantiates a new Bare worklet on a worker thread.

Options include:

```typescript
  assets: string,
  memoryLimit: number,
```

`Worklet` is also an event emitter. Available events include:

```typescript
event: start | terminate | suspend | resume | wakeup
```

#### `await Worklet.suspend(linger)`

Suspend all worklets stopping all I/O. It's important to suspend the worklets when app enters background, because on mobile operating system might kill the app otherwise.

```typescript
// How much time the worklets have to stop I/O
linger: number
```

#### `await Worklet.resume()`

Exit suspended state and resume I/O for all worklets.

#### `await Worklet.wakeup(deadline)`

Temporalily wake suspended worklets to perform additional work (e.g. handle a notification).

```typescript
// Time allocated to perform given task
deadline: number
```

#### `worklet.IPC`

The bidirectional IPC interface. It inherits from `streamx.Duplex` and has a similar API to node streams.

#### `worklet.started`

`boolean` indicating if `worklet.start(...)` has been called.

#### `worklet.terminated`

`boolean` indicating if `worklet.terminate()` has been called.

#### `worklet.suspended`

`boolean` indicating current state of the worklet.

#### `await worklet.start(filename, source, args)`

Start the worklet. After this method succeeds IPC becomes available.

Arguments are:

```typescript
// Either a file on disk to use as source or dummy path to indicate the type of source
// It should always end in .bundle
filename: string
// The source to execute. If you're providing a real path to filename argument
// set this to null
source: string | Uint8Array | null
// The command line flags to pass to Bare/V8
args: string[]
```

You should provide dummy path to `filename` such as `/app.bundle` and actual sources through `source` argument if you want to support mobile. Reading sources from filesystem can be problematic, because such access is restricted and could potentially violate app store policies.

#### `await worklet.suspend(linger)`

Suspend the worklet stopping all I/O. It's important to suspend the worklet when app enters background, because on mobile operating system might kill the app otherwise.

```typescript
// How much time the worklet has to stop I/O
linger: number
```

#### `await worklet.resume()`

Exit suspended state and resume I/O.

#### `await worklet.wakeup(deadline)`

Temporalily wake suspended worklet to perform additional work (e.g. handle a notification).

```typescript
// Time allocated to perform given task
deadline: number
```

#### `await worklet.terminate()`

Stop and destroy worklet freeing resources.

### Logging

BareKit uses [liblog](https://github.com/holepunchto/liblog) as logging provider.
You can read on how to access the log there. `tauri-plugin-bare-kit` logs with `bare-kit` as an identifier.

## License

Apache-2.0
