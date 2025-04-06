use super::world_holder::WorldHolder;

pub fn generate_ores( world_holder:&mut dyn WorldHolder, from:(u32, u32, u32), to:(u32, u32, u32) ) {
    // Ore::Coal.generate( world_holder );
}

enum Ore {
    Coal,
}

impl Ore {
    pub fn generate( &self, world_holder:&mut dyn WorldHolder, from:(u32, u32, u32), to:(u32, u32, u32) ) {
        match self {
            Ore::Coal => {
                // let origins =
            }
        }
    }
}
