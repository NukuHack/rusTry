
# Project TODO  

## High Priority  
- [x] slight ui rework (mainly just make the text not overflow)  
- [x] making text wrap if it's too long  
- [x] fly and general movement rework  
- [x] fov and general camera rework (also drag nice-ify)  
- [x] making the save-load work  
- [x] world gen from noise  
- [ ] settings for sound volume font and stuff  
- [x] refine inventory and item and block things  
- [ ] item and block things correctly  
- [ ] chunk meshing correction - neighbors  
- [ ] making the save-load auto trigger (actually load the world on startup)  
- [ ] correcting the save to be world wise not just player pos relative 
- [ ] Ambient oc. and other lighting  

## Low Priority  
- [x] 2d texture array and load in some other textures -> more blocks  
- [x] New UI element as Slider  
- [x] A "multi-state" button kind of thing (like a slider but with a button)  
- [ ] hardcode the "missing texture" texture (purple black or idk)  
- [ ] Some basic Rounding for UI elements (maybe implement some neat stuff inside the shader)  
- [ ] Making a new "panel" kind of thing with scrollable contents  


- [ ] Maybe make the UIelements be centered around their pos and not just the pos being a corner ?  

## Compiled stuff

Technique	Purpose	Example 	Usage  
Blinn-Phong	Cheap lighting		Wooden crates, character skin  
SMAA/FXAA	Smooth edges		Post-process pass  
Baked Lighting	Static shadows		Buildings, terrain  
Texture Atlases	Reduce draw calls	Foliage, debris  
Fog		Hide LOD transitions	Dungeons, open worlds  
 
## Textures (for realistic lights and heights)  
Albedo (RGBA) – Base texture.  
Normal (three channel).  
Bump (one channel).  
Roughness (one channel).  
Metallic (one channel).  

## Actual lighting (algorythms)  
Physically Based Rendering (PBR) (currently the "best", actually physics based)  
Blinn-Phong (BP) (one of the earliest from late 20th century, pretty cheap too)  
[link](https://google.github.io/filament/Filament.md.html) - middle ?  
