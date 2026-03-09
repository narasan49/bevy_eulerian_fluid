#define_import_path bevy_fluid::hash

fn rotl32(x: vec2<u32>) -> vec2<u32> {
    return (x << vec2<u32>(15)) | (x >> vec2<u32>(17));
}

fn murmurHash3_22(src: vec2<u32>) -> vec2<u32> {
    const seed: vec2<u32> = vec2<u32>(12345, 98765);
    const c1: u32 = 0xcc9e2d51u;
    const c2: u32 = 0x1b873593u;

    var k = src;
    var h = seed;
    
    const nblocks: i32 = 1;
    k *= c1;
    k = rotl32(k);
    k *= c2;
    
    h ^= k;
    
    h = rotl32(h);
    h = h * 5u + 0xe6546b64u;
    
    h ^= vec2<u32>(4);
    h ^= h >> vec2<u32>(16);
    h *= 0x85ebca6bu;
    h ^= h >> vec2<u32>(13);
    h *= 0xc2b2ae35u;
    h ^= h >> vec2<u32>(16);
    
    return h;
}

fn hash22(value: vec2<f32>) -> vec2<f32> {
    let hash = murmurHash3_22(bitcast<vec2<u32>>(value));
    return bitcast<vec2<f32>>(hash & vec2<u32>(0x007fffffu) | vec2<u32>(0x3f800000u)) - 1.0;
}