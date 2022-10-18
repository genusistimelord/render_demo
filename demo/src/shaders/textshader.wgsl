struct Camera {
    view_proj: mat4x4<f32>,
    eye: vec3<f32>,
};

struct Time {
    seconds: f32,
};

struct ScreenResolution {
    width: u32,
    height: u32
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<uniform> time: Time;

@group(2)
@binding(0)
var<uniform> resolution: ScreenResolution;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec3<f32>,
    @location(2) color: vec4<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec3<f32>,
    @location(2) size: vec2<f32>,
};

@group(3)
@binding(0)
var tex: texture_2d_array<f32>;
@group(3)
@binding(1)
var tex_sample: sampler;

@vertex
fn vertex(
    vertex: VertexInput,
) -> VertexOutput {
    var result: VertexOutput;
    var pos = vertex.pos;
    var uv = vec2<f32>(vertex.uv.x, vertex.uv.y);
    let color = vertex.color;
    let size = textureDimensions(tex);
    let fsize = vec2<f32> (f32(size.x), f32(size.y));

    result.position =  camera.view_proj * vec4<f32>(vertex.pos.xyz, 1.0);
    result.size = fsize;
    result.color = vec4<f32>(
        1.0,
        1.0,
        1.0,
        1.0,
    ) ;

    var uv  = vec2<f32>(uv) /  fsize;
    result.uv = vec3<f32>(uv.x, uv.y, vertex.uv.z);
    return result;
}

// Fragment shader
@fragment
fn fragment(vertex: VertexOutput,) -> @location(0) vec4<f32> {
    let coords = vec2<f32>(vertex.uv.x, vertex.uv.y);
    
    var step = vec2<f32>(0.5, 0.5);
    var tex_pixel = vertex.size * coords - step.xy / 2.0;

    let corner = floor(tex_pixel) + 1.0;
    let frac = min((corner - tex_pixel) * vec2<f32>(2.0, 2.0), vec2<f32>(1.0, 1.0));

    var c1 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, 0.0)) + 0.5) / vertex.size, i32(vertex.uv.z));
    var c2 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(step.x, 0.0)) + 0.5) / vertex.size, i32(vertex.uv.z));
    var c3 = textureSample(tex, tex_sample, (floor(tex_pixel + vec2<f32>(0.0, step.y)) + 0.5) / vertex.size, i32(vertex.uv.z));
    var c4 = textureSample(tex, tex_sample, (floor(tex_pixel + step.xy) + 0.5) / vertex.size, i32(vertex.uv.z));

    c1 = c1 * (frac.x * frac.y);
    c2 = c2 *((1.0 - frac.x) * frac.y);
    c3 = c3 * (frac.x * (1.0 - frac.y));
    c4 = c4 *((1.0 - frac.x) * (1.0 - frac.y));

    let object_color = (c1 + c2 + c3 + c4);
    
    //let r = textureSample(tex, tex_sample, vertex.uv.xy, i32(vertex.uv.z)).r;
    if object_color.r <= 0.0 {
        discard;
    }
    return vertex.color.rgba * object_color.r;
}