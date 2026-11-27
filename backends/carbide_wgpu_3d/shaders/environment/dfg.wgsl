// The DFG lookup table of the split-sum approximation (Karis, "Real Shading in Unreal Engine 4"):
// for N·V across and perceptual roughness down, the scale (red) and bias (green) that turn F0 into
// the specular BRDF's integral over the hemisphere, F0 · scale + bias. Uses the height-correlated
// Smith visibility, as the lights' shading (`pbr/brdf.wgsl`) and Filament.
//!include environment/face.wgsl

const DFG_SIZE = 128.0;
const DFG_SAMPLES = 1024u;

fn v_smith_ggx_correlated(nov: f32, nol: f32, alpha: f32) -> f32 {
    let a2 = alpha * alpha;
    let ggxv = nol * sqrt((nov - a2 * nov) * nov + a2);
    let ggxl = nov * sqrt((nol - a2 * nol) * nol + a2);
    return 0.5 / (ggxl + ggxv);
}

@fragment
fn dfg_fs(in: FaceOutput) -> @location(0) vec4<f32> {
    let uv = in.position.xy / DFG_SIZE;
    let nov = uv.x;
    let alpha = uv.y * uv.y;
    let v = vec3<f32>(sqrt(1.0 - nov * nov), 0.0, nov);

    var scale = 0.0;
    var bias = 0.0;
    for (var i = 0u; i < DFG_SAMPLES; i++) {
        let h = importance_sample_ggx(hammersley(i, DFG_SAMPLES), alpha);
        let l = 2.0 * dot(v, h) * h - v;
        let nol = l.z;
        let noh = h.z;
        let voh = max(dot(v, h), 0.0);
        if (nol > 0.0) {
            // The visibility times the sample's weight: D cancels with the pdf, D · N·H / (4 V·H).
            let g = v_smith_ggx_correlated(nov, nol, alpha) * 4.0 * nol * voh / noh;
            let fc = pow(1.0 - voh, 5.0);
            scale += (1.0 - fc) * g;
            bias += fc * g;
        }
    }

    return vec4<f32>(scale / f32(DFG_SAMPLES), bias / f32(DFG_SAMPLES), 0.0, 1.0);
}
