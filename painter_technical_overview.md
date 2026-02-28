# trivalibs_painter — Technical Overview for WebGPU Reimplementation

This document describes the architecture, abstractions, and design decisions of `trivalibs_painter` for someone who wants to reimplement the same concepts in a different language (e.g. TypeScript/JavaScript) targeting the WebGPU browser API. The focus is on _what_ the abstractions do and _why_, not on Rust-specific implementation details.

---

## Motivation

The WebGPU API is low-level by design. Rendering a single triangle requires: creating a shader module, specifying a vertex buffer layout, building a bind group layout, creating a pipeline layout, creating a render pipeline, allocating a vertex buffer, writing data to it, creating a command encoder, beginning a render pass, setting the pipeline, setting the vertex buffer, drawing, ending the pass, and submitting to the queue.

For iterative creative work — building generative graphics, visual effects, or interactive simulations — this boilerplate is the enemy. `trivalibs_painter` collapses this into five composable primitives:

- **Shade**: the shader program and its input contract
- **Form**: the geometry data on the GPU
- **Shape**: a drawable object (Form + Shade + data bindings)
- **Effect**: a fullscreen post-processing pass — a Shape specialization that requires no Form and no vertex shader; the geometry is always the full viewport quad, driven by a built-in vertex shader
- **Layer**: a render target containing an ordered list of Shapes and Effects to render

**Shape and Effect share the same abstraction for bindings, blend state, and instancing.** The only difference is that a Shape requires explicit geometry (a Form) and a full vertex+fragment Shade, while an Effect replaces both with the implicit fullscreen quad and a fragment-only Shade. This makes Effects a natural, first-class primitive rather than a special-cased feature of Layer.

These map cleanly to WebGPU concepts but at a much higher level of abstraction.

---

## Core Design Principles

Before describing each abstraction, it helps to understand the principles that govern the whole system.

### 1. Handles, not objects

Every GPU resource is represented by a lightweight **handle** — an opaque identifier (an integer index) rather than a rich object. All actual GPU resources (buffers, textures, pipelines, bind groups) live in a central **Painter** registry.

This means handles are cheap to copy and store anywhere. There are no reference counts, no lifetimes, no ownership concerns in user code. The Painter owns everything.

### 2. Builders for creation

All resources are created through a **builder pattern**: you configure a resource with a chain of method calls, then call `.create()` to finalize it and get back a handle. After creation, resources are not reconfigured — you update _data_ (write new values to GPU buffers) but not _structure_ (the pipeline layout, attribute format, etc.).

### 3. Lazy pipeline creation

WebGPU `GPURenderPipeline` objects are expensive to create. The Painter creates them on demand (at first render) and caches them, keyed by a hash of their configuration (shader + blend state + topology + cull mode). Multiple Shapes that share the same configuration reuse the same pipeline.

### 4. Declarative rendering

You declare _what_ to render (shapes in a layer) and _how_ (bindings, blend state, multisampling). The Painter executes the actual render passes. The frame loop is simply:

```
update data → painter.paint(layer) → painter.show(layer)
```

### 5. Binding hierarchy

Data is passed to shaders at three levels, from lowest to highest priority:

```
Layer bindings  →  Shape/Effect bindings  →  Instance bindings
```

Higher levels override lower levels at the same binding slot. This lets you share a view-projection matrix at the layer level and override only the model matrix per shape.

---

## The Painter

The `Painter` is the central registry and execution engine. It holds:

- The WebGPU device and queue
- Storage arrays for all resources (shades, forms, shapes, layers, effects, buffers, textures, samplers, bind groups)
- A cache of `GPURenderPipeline` objects
- The surface / canvas for display

All builder methods are on the Painter: `painter.shade(...)`, `painter.form(...)`, `painter.shape(...)`, `painter.layer()`, etc.

**In a JavaScript/TypeScript reimplementation**, the Painter would be a class holding a `GPUDevice`, `GPUQueue`, and arrays of stored resources. Handles would simply be integer indices into those arrays.

```typescript
class Painter {
  device: GPUDevice;
  queue: GPUQueue;
  canvas: HTMLCanvasElement;
  context: GPUCanvasContext;

  // Resource storage
  shades: ShadeStorage[];
  forms: FormStorage[];
  shapes: ShapeStorage[];
  layers: LayerStorage[];
  effects: EffectStorage[];
  buffers: GPUBuffer[];
  textures: TextureStorage[];
  samplers: GPUSampler[];
  bindGroups: BindGroupStorage[];

  // Pipeline cache
  pipelines: Map<string, GPURenderPipeline>;
}
```

---

## Shade — The Shader Program and Its Contract

### Concept

A `Shade` defines:

1. The **vertex attribute layout**: what data each vertex carries (position, normal, UV, etc.) and how it's packed in memory
2. The **binding layout**: what uniform buffers, samplers, and input textures the shader expects, at which slots, and in which shader stage (vertex or fragment)
3. The **shader code** itself

This is a combination of WebGPU's `GPUShaderModule`, `GPUBindGroupLayout`, and `GPUVertexBufferLayout`.

The key insight is that the Shade defines a _contract_: any Shape using this Shade must supply data matching these layouts. The Shade is reusable across many Shapes.

### Vertex Attributes

Attributes are described as an ordered list of `GPUVertexFormat` values. The Shade automatically computes the stride (total bytes per vertex) and assigns each attribute a sequential shader location.

For example, `[Float32x3, Float32x3, Float32x2]` means:

- Location 0: `vec3f` at offset 0 (12 bytes)
- Location 1: `vec3f` at offset 12 (12 bytes)
- Location 2: `vec2f` at offset 24 (8 bytes)
- Stride: 44 bytes

This becomes the `GPUVertexBufferLayout` passed to the pipeline descriptor.

### Binding Layouts

Bindings are declared as an ordered list of binding types, each producing a `GPUBindGroupLayoutEntry`. Supported types include:

- `BINDING_BUFFER_VERT` — uniform buffer, visible in vertex shader
- `BINDING_BUFFER_FRAG` — uniform buffer, visible in fragment shader
- `BINDING_BUFFER_VERT_FRAG` — uniform buffer, visible in both
- `BINDING_SAMPLER_FRAG` — sampler, visible in fragment shader

**Layer bindings** (input textures from other layers) are declared separately from value bindings (buffers and samplers). This separation is important: layer bindings may need to change every frame (as the source layer's texture changes between passes), while value bindings are typically stable.

The Shade creates two `GPUBindGroupLayout` objects: one for value bindings, one for layer bindings. These are combined into a `GPUPipelineLayout`.

### Effect Shades

A special variant of Shade — the _effect shade_ — has no vertex attributes. It is used for fullscreen post-processing passes, which always operate on a built-in fullscreen triangle. Effect shades only need a fragment shader.

### Shader Code

In the Rust implementation, shaders are compiled SPIR-V loaded at runtime. For a WebGPU reimplementation, shaders would be WGSL strings (or modules) assigned separately after creating the Shade handle.

### Storage

```typescript
interface ShadeStorage {
  vertexModule: GPUShaderModule | null;
  fragmentModule: GPUShaderModule;
  attribsFormat: GPUVertexBufferLayout; // null for effect shades
  valueBindGroupLayout: GPUBindGroupLayout | null;
  layerBindGroupLayout: GPUBindGroupLayout | null;
  pipelineLayout: GPUPipelineLayout;
  valueBindingCount: number;
  layerBindingCount: number;
}

type Shade = number; // index into painter.shades
```

---

## Form — Geometry on the GPU

### Concept

A `Form` is the raw geometry data: a vertex buffer and an optional index buffer, along with the primitive topology (triangle list, triangle strip, line list, etc.) and front-face winding.

The Form is the answer to "what geometry do I draw?" It is independent of any shader — the same Form could be drawn with different Shades.

### Creating a Form

The user provides vertex data as a typed array (or `ArrayBuffer`). The Painter allocates a `GPUBuffer` with `VERTEX` usage and writes the data. If index data is provided, an additional buffer with `INDEX` usage is created.

Key implementation detail: WebGPU requires buffer sizes to be multiples of 4. The Rust implementation pads to 256 bytes (WGPU's minimum uniform binding size), but for vertex buffers the requirement is just 4-byte alignment.

### Dynamic Geometry

Forms support updating their data after creation. If the new data fits in the existing buffer, the buffer is reused (via `device.queue.writeBuffer`). If the new data is larger, the old buffer is destroyed and a new one is allocated.

For animations or procedurally generated geometry, you call `form.update(painter, newVertexData)` each frame.

### Storage

```typescript
interface FormGPUBuffers {
  vertexBuffer: GPUBuffer;
  vertexBufferSize: number; // currently allocated size
  vertexCount: number;
  indexBuffer: GPUBuffer | null;
  indexBufferSize: number;
  indexCount: number;
}

interface FormStorage {
  buffers: FormGPUBuffers[]; // supports multiple buffer sets
  activeBufferCount: number;
  topology: GPUPrimitiveTopology;
  frontFace: GPUFrontFace;
}

type Form = number; // index into painter.forms
```

---

## Shape — A Drawable Object

### Concept

A `Shape` combines:

- A **Form** (what geometry)
- A **Shade** (what shader program)
- **Value bindings**: the uniform buffers and samplers to pass to the shader
- **Layer bindings**: textures from other layers to use as shader inputs
- **Rendering state**: cull mode, blend state
- Optionally, **instances**: a list of per-draw-call binding overrides

The Shape is the unit of rendering. When the Painter renders a Layer, it iterates over the Layer's Shapes and draws each one.

### Pipeline Key

The render pipeline for a Shape is determined by the combination of:

- Which Shade (determines shader code and layouts)
- Blend state (alpha blend, additive, replace, etc.)
- Cull mode
- Primitive topology (from the Form)

These are serialized into a string (or hash) that serves as the cache key. If another Shape would produce the same key, they share the same `GPURenderPipeline`.

### Bind Groups

At render time (or at first paint, for initialization), the Painter resolves the Shape's bindings into `GPUBindGroup` objects:

1. One bind group for value bindings (buffers + samplers)
2. One bind group for layer bindings (textures from other layers)

These are created from the merged binding data: layer-level bindings are the base, shape-level bindings override at matching slots, instance-level bindings override further.

### Instance Rendering

"Instancing" in this system means rendering the same Shape multiple times with different bindings per draw call — not GPU-level instancing with instance buffers.

If a Shape has N instances, the Painter issues N draw calls, setting different bind groups each time.

The Painter optimizes this based on which binding categories vary:

| What varies            | Strategy                                                       |
| ---------------------- | -------------------------------------------------------------- |
| Nothing (no instances) | Set bindings once, one draw call                               |
| Only value bindings    | Set layer bind group once; loop updating only value bind group |
| Only layer bindings    | Set value bind group once; loop updating only layer bind group |
| Both                   | Update both bind groups per draw call                          |

This minimizes the number of `setBindGroup` calls on the render pass encoder.

### Storage

```typescript
interface ShapeStorage {
  form: Form;
  shade: Shade;
  valueBindings: [number, ValueBinding][]; // slot → binding
  layerBindings: [number, LayerBinding][]; // slot → layer reference
  instances: InstanceBinding[];
  cullMode: GPUCullMode;
  blendState: GPUBlendState;
  pipelineKey: string;
}

type Shape = number; // index into painter.shapes
```

---

## Layer — Render Target and Composition

### Concept

A `Layer` is the highest-level abstraction. It serves two roles:

1. **Render target**: it owns the GPU textures that shapes and effects render into
2. **Composition unit**: it holds an ordered list of Shapes and Effects to execute

Rendering a Layer means:

1. Begin a render pass targeting the layer's textures
2. Clear (if configured)
3. Draw each Shape in order
4. End the render pass
5. For each Effect: begin a new render pass reading from the previous output, draw a fullscreen quad, end the render pass
6. Generate mipmaps (if configured)

The output of a rendered Layer is its texture(s), which can be bound as input to Shapes or Effects in other Layers.

### Target Textures

By default, a Layer has a single RGBA texture with a standard format. The size defaults to the window/canvas dimensions and updates automatically on resize.

Custom sizes are supported for off-screen rendering (shadow maps, render-to-texture, etc.).

**Multiple Render Targets (MRT)**: A Layer can render simultaneously to multiple textures with different formats. This is essential for deferred rendering (G-buffers):

```
Layer with formats [RGBA8, RGBA16Float, RGBA16Float]
  → three GPUTextures written in parallel by fragment shader
  → accessible as layer.bindingAt(0), layer.bindingAt(1), layer.bindingAt(2)
```

Implementing MRT requires the render pass `colorAttachments` to include all target textures, and the fragment shader to output to multiple locations.

### Optional Features per Layer

- **Depth testing**: creates a `depth24plus` texture attached as `depthStencilAttachment`
- **MSAA (multisampling)**: creates additional multisampled textures; the render pass targets the multisampled texture and resolves to the regular texture. In WebGPU, this is the `resolveTarget` in `colorAttachments`.
- **Mipmaps**: after rendering, generate mipmaps for the output texture. Can be done with a compute shader or a series of blit passes.
- **Static texture**: initialize a Layer from image data; it is never re-rendered. Useful for texture assets.

### Shared Bindings

A Layer can hold default bindings shared by all its Shapes and Effects. This is useful for per-frame data like a view-projection matrix: set it once on the Layer, and every Shape in the Layer receives it without per-Shape configuration.

```typescript
interface LayerStorage {
  shapes: ShapeData[];
  effects: EffectData[];
  targetTextures: TextureStorage[];
  depthTexture: TextureStorage | null;
  msaaTextures: TextureStorage[] | null;
  width: number;
  height: number;
  useWindowSize: boolean;
  clearColor: GPUColor | null;
  formats: GPUTextureFormat[];
  multisampled: boolean;
  depthTest: boolean;
  mips: MipMapCount | null;
  valueBindings: [number, ValueBinding][];
  layerBindings: [number, LayerBinding][];
}

type Layer = number; // index into painter.layers
```

---

## Effect — Post-Processing Pass

### Concept

An `Effect` is a Layer's post-processing step. It renders a fullscreen quad (or triangle) using a fragment shader that reads from the Layer's current output texture and writes to a new one.

An Effect is essentially a Shape without any Form: its geometry is always a hardcoded fullscreen triangle (three vertices covering the entire viewport, no vertex buffer needed). Only a fragment shader is required.

### The Fullscreen Triangle

The standard technique is a single triangle that covers the clip-space viewport, generated entirely in the vertex shader from `vertex_index`:

```wgsl
// WGSL vertex shader for fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
  let x = f32((vi << 1u) & 2u) * 2.0 - 1.0;
  let y = f32(vi & 2u) * 2.0 - 1.0;
  return vec4f(x, y, 0.0, 1.0);
}
```

This vertex shader is built into the Painter and shared by all Effects. No vertex buffer is created.

### Target Swapping (Ping-Pong)

When a Layer has multiple Effects, the output of one Effect becomes the input of the next. This is implemented with texture ping-pong:

- The Layer has two sets of target textures (A and B)
- Shapes render into A
- Effect 1: reads A → writes B
- Effect 2: reads B → writes A
- Effect 3: reads A → writes B
- ...

The "current" texture (the one other Layers bind when they reference this Layer) is whichever was last written to.

Implementation: the Layer tracks a `currentTarget` index (0 or 1), toggling it after each Effect pass. When an Effect is done, the swap happens unless the Effect writes to a specific mip level (in which case no swap is needed — the effect wrote into an auxiliary mip, not the main target).

### Effect Instances

Effects support the same instance mechanism as Shapes: multiple draw calls with per-instance binding overrides. This is the key technique for **deferred lighting**: a single fullscreen lighting shader is run once per light, with additive blending, using per-instance uniform data for light position and color.

```
for each light:
  set blend state: additive
  set bindings: { 2: light.positionBuffer, 3: light.colorBuffer }
  draw fullscreen quad
```

The result accumulates light contributions in the output texture.

### Mip Level Targeting

Effects can read from a specific mip level of a layer (`srcMipLevel`) and write to a specific mip level (`dstMipLevel`). This enables:

- Manual mipmap generation with custom filtering
- Hierarchical blur (reading coarser mips to spread blur efficiently)
- Progressive downsampling chains

---

## The Binding System

### Value Bindings

Value bindings wrap GPU uniform buffers and samplers. They are typed wrappers that handle:

- Allocating the `GPUBuffer` with `UNIFORM | COPY_DST` usage
- Writing data with `queue.writeBuffer()`
- Providing a `ValueBinding` reference for use in Shapes/Effects/Layers

In WebGPU, uniform buffers have alignment requirements:

- Buffer size must be a multiple of 256 bytes (for `minUniformBufferOffsetAlignment`)
- `vec3` types need 16-byte alignment (pad to `vec4` in practice)

**Common binding types to support:**

- Scalar: `f32`, `u32`, `i32`
- Vector: `vec2f`, `vec3f` (padded to `vec4f`), `vec4f`
- Matrix: `mat4x4f`, `mat3x3f` (padded to `mat4x4f`)
- Sampler: wraps a `GPUSampler`

For mutable bindings, the user holds a reference and calls `.update(painter, newValue)` to write new data to the GPU each frame.

For constant bindings (values that never change), the Painter allocates a buffer, writes the initial value, and returns a `ValueBinding` directly — the user does not need to keep a reference.

### Layer Bindings

Layer bindings expose a Layer's rendered texture(s) as a `GPUTextureView` for use in shaders. Variants:

- `Source(layer)` — the layer's current output texture view
- `AtIndex(layer, i)` — a specific MRT target (for G-buffers)
- `SourceAtMipLevel(layer, mipLevel)` — a specific mip level view
- `Depth(layer)` — the layer's depth texture view

Layer bindings are resolved fresh each render call, because the underlying texture can change (ping-pong targets swap after each Effect).

### Bind Group Construction

When rendering, the Painter constructs `GPUBindGroup` objects from the resolved bindings. The process:

1. Start with layer-level bindings as the base
2. Override with shape/effect-level bindings at matching slots
3. For instance rendering, override further with per-instance bindings

Bind groups can be cached per-frame, but must be re-created if any layer binding references a texture that changed (due to ping-pong). Value binding bind groups are more stable and can be cached across frames.

---

## The Rendering Loop

### Painting a Layer

```
painter.paint(layer):
  1. If layer.msaa: target = layer.msaaTextures, resolve = layer.targetTextures
     Else: target = layer.targetTextures

  2. Create render pass:
     colorAttachments: target textures (+ resolve targets for MSAA)
     depthStencilAttachment: layer.depthTexture (if any)
     clearValues: layer.clearColor

  3. For each shape in layer.shapes:
     a. Get or create GPURenderPipeline (from cache by pipelineKey)
     b. Resolve bind groups (merge layer + shape + instance bindings)
     c. set pipeline
     d. set vertex buffer (form.vertexBuffer)
     e. set index buffer (form.indexBuffer, if any)
     f. For each instance (or once if no instances):
        - setBindGroup(0, valueBindGroup)
        - setBindGroup(1, layerBindGroup)
        - draw(vertexCount) or drawIndexed(indexCount)

  4. End render pass, submit command buffer

  5. For each effect in layer.effects:
     a. Swap active target (ping-pong)
     b. Resolve source texture = previous target
     c. Create new render pass targeting new active target
     d. Get or create GPURenderPipeline
     e. Resolve bind groups (including source texture as layer binding)
     f. For each instance (or once):
        - setBindGroup(0, valueBindGroup)
        - setBindGroup(1, layerBindGroup)
        - draw(3)  // fullscreen triangle
     g. End render pass, submit
     (unless effect writes to mip level: no swap, no re-submission needed)

  6. If layer.mips: generate mipmaps for target textures
```

### Showing a Layer

To display a Layer to the screen:

```
painter.show(layer):
  1. Get current swap chain texture: context.getCurrentTexture()
  2. Create render pass targeting the swap chain texture
  3. Run a fullscreen blit effect: draw the layer's texture to screen
  4. End render pass, submit
  5. context.present() (if needed by platform)
```

The blit is a minimal fullscreen triangle with a simple fragment shader that samples the layer texture.

### Composing Multiple Layers

For multi-pass rendering, the user calls `paint()` on each layer in dependency order before `show()`:

```
// Deferred rendering frame:
painter.paint(sceneLayer);    // render G-buffer
painter.paint(lightingLayer); // accumulate lighting using G-buffer
painter.show(lightingLayer);  // display result
```

The Painter provides a `compose(layers)` helper that paints all layers in order, equivalent to calling `paint()` on each.

---

## Application Framework

The Painter includes a minimal application loop wrapper. In a browser context, this is straightforward:

```typescript
interface CanvasApp {
  init(painter: Painter): void;
  frame(painter: Painter, deltaTime: number): void;
  resize(painter: Painter, width: number, height: number): void;
  event(event: Event, painter: Painter): void;
}
```

The framework handles:

- Requesting a WebGPU adapter and device
- Creating the canvas context
- Setting up a `ResizeObserver` to call `resize()` and update layer dimensions
- Driving the render loop with `requestAnimationFrame`
- Computing `deltaTime` (time per frame in seconds)

`painter.requestNextFrame()` signals that the app wants continuous animation. If not called, rendering stops until the next user event — useful for static or event-driven renders.

---

## WebGPU-Specific Implementation Notes

### Adapter and Device Initialization

```typescript
const adapter = await navigator.gpu.requestAdapter();
const device = await adapter.requestDevice();
const context = canvas.getContext("webgpu");
const format = navigator.gpu.getPreferredCanvasFormat();
context.configure({ device, format });
```

### Uniform Buffer Alignment

WebGPU enforces `minUniformBufferOffsetAlignment` (typically 256 bytes). All uniform buffers must be padded to a multiple of 256 bytes:

```typescript
function paddedSize(size: number): number {
  return Math.ceil(size / 256) * 256;
}
```

Additionally, `vec3f` in WGSL has 16-byte alignment. Pass `vec3` values padded to `vec4` in the buffer, or use a struct with explicit padding.

### Pipeline Creation

```typescript
const pipeline = device.createRenderPipeline({
  layout: shade.pipelineLayout,
  vertex: {
    module: shade.vertexModule,
    entryPoint: "vs_main",
    buffers: [shade.attribsFormat],
  },
  fragment: {
    module: shade.fragmentModule,
    entryPoint: "fs_main",
    targets: layer.formats.map((format) => ({
      format,
      blend: shape.blendState,
    })),
  },
  primitive: {
    topology: form.topology,
    frontFace: form.frontFace,
    cullMode: shape.cullMode,
  },
  depthStencil: layer.depthTest
    ? {
        format: "depth24plus",
        depthWriteEnabled: true,
        depthCompare: "less",
      }
    : undefined,
  multisample: layer.multisampled ? { count: 4 } : undefined,
});
```

### MSAA in WebGPU

WebGPU MSAA uses sample count 4. The multisampled texture is the render target; the regular texture is the resolve target:

```typescript
colorAttachments: [
  {
    view: msaaTextureView, // render here (multisampled)
    resolveTarget: textureView, // resolve MSAA to this
    clearValue: layer.clearColor,
    loadOp: "clear",
    storeOp: "discard", // MSAA texture is discarded after resolve
  },
];
```

### Mipmap Generation

WebGPU does not have a built-in mipmap generation call. Options:

1. A series of render passes, each reading mip N and writing mip N+1, using a blit shader
2. A compute shader that downsamples multiple mips in one pass

The render pass approach is simpler to implement and works on all WebGPU-capable devices.

### Shader Hot Reloading

In a browser context, hot reloading can be implemented using WebSockets connected to a dev server that watches shader files. When a file changes, the server sends the new WGSL source; the client recreates the shader module and invalidates cached pipelines for that Shade.

---

## Summary: Abstraction Layers

```
WebGPU API
  ↑
Painter (resource registry + render execution)
  ↑
┌──────────────────────────────────────────────────────────────────────┐
│  Shade    │  Form     │  Shape         │  Effect        │  Layer     │
│ (shader   │ (vertex   │ (Form + Shade  │ (Shade only,   │ (render    │
│  program  │  buffers) │  + bindings)   │  fullscreen    │  target +  │
│  + layout)│           │                │  quad, no Form)│  drawlist) │
└──────────────────────────────────────────────────────────────────────┘
  ↑
CanvasApp (frame loop + resize + events)
```

Each abstraction has a single responsibility, and they compose cleanly:

- **Shade** answers: "What shader runs and what inputs does it need?"
- **Form** answers: "What geometry is on the GPU?"
- **Shape** answers: "What do I draw and with what data?" — requires both a Form and a Shade
- **Effect** answers: "What do I apply to a layer's output?" — a Shape specialization: same binding system, same blend control, same instancing, but the geometry is always the full viewport quad and only a fragment shader is needed
- **Layer** answers: "Where do I render, in what order, and with what shared data?"
- **Painter** answers: "How do I execute all of this efficiently?"
