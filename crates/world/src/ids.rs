//! Content ids of the world: string newtypes, one per kind, so a hero id can
//! never be passed where a monster id is expected.

use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! content_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[cfg_attr(feature = "ts", derive(ts_rs::TS))]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wraps a content id.
            #[must_use]
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// The id as written in content files.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

content_id!(
    /// A piece of the world map (`brookhollow`, `thornwood`…).
    RegionId
);
content_id!(
    /// An instance: the close-up map of an area (`thornwood-road`).
    InstanceId
);
content_id!(
    /// A point of an instance, unique inside it.
    NodeId
);
content_id!(
    /// A story flag (`lyra-joined`); set once, never cleared.
    Flag
);
content_id!(
    /// A story hero (`taren`).
    HeroId
);
content_id!(
    /// A hero class (`warrior`).
    ClassId
);
content_id!(
    /// A monster kind (`forest-wolf`).
    MonsterId
);
content_id!(
    /// A dialogue scene.
    DialogueId
);
content_id!(
    /// A shop's stock.
    ShopId
);
content_id!(
    /// A weapon or armor.
    EquipmentId
);
content_id!(
    /// A mission (`sewer-rats`).
    QuestId
);

/// Where the player stands: a node of an instance.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Position {
    /// The instance.
    pub instance: InstanceId,
    /// The node inside it.
    pub node: NodeId,
}
