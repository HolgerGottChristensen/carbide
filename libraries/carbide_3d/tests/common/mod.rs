//! Helpers shared by the rendering tests in this folder. Each test file is its own crate and
//! declares `mod common;`, so not every file uses every helper.
#![allow(dead_code)]

// Linking the wgpu 3D backend registers its plugin, so scenes in these tests render with it.
use carbide_wgpu_3d as _;
