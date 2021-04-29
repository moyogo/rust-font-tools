///! OpenType Variations common tables
mod itemvariationstore;
mod packeddeltas;
mod packedpoints;
mod tuplevariationheader;
mod tuplevariationstore;

use otspec::types::int16;

/// Represents either a two-dimensional (gvar) or one-dimensional (cvt) delta value
#[derive(Debug, PartialEq)]
pub enum Delta {
    Delta1D(int16),
    Delta2D((int16, int16)),
}
impl Delta {
    /// Assuming that this is a two-dimensional delta, returns the delta as a
    /// X,Y coordinate tuple.
    pub fn get_2d(&self) -> (int16, int16) {
        if let Delta::Delta2D(p) = self {
            *p
        } else {
            panic!("Tried to turn a scalar delta into a coordinate delta");
        }
    }
}
pub use crate::otvar::itemvariationstore::{
    ItemVariationData, ItemVariationStore, RegionAxisCoordinates,
};
pub use crate::otvar::packeddeltas::{PackedDeltas, PackedDeltasDeserializer};
pub use crate::otvar::packedpoints::PackedPoints;
pub use crate::otvar::tuplevariationheader::{
    TupleIndexFlags, TupleVariationHeader, TupleVariationHeaderDeserializer,
};
pub use crate::otvar::tuplevariationstore::{TupleVariation, TupleVariationStoreDeserializer};

#[cfg(test)]
mod tests {
    use crate::otvar;

    #[test]
    fn otvar_de_ivd() {
        let binary_ivd = vec![
            0x00, 0x04, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0x38, 0xFF, 0xCE, 0x00, 0x64,
            0x00, 0xC8,
        ];
        let fivd = otvar::ItemVariationData {
            regionIndexes: vec![0],
            deltaValues: vec![vec![-200], vec![-50], vec![100], vec![200]],
        };
        let deserialized: otvar::ItemVariationData = otspec::de::from_bytes(&binary_ivd).unwrap();
        assert_eq!(deserialized, fivd);
    }

    #[test]
    fn otvar_de_ivs() {
        let binary_ivs = vec![
            0x00, 0x01, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x01, 0x00, 0x00, 0x00, 0x16, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00, 0x40, 0x00, 0x40, 0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x01,
            0x00, 0x00, 0xFF, 0x38, 0xFF, 0xCE, 0x00, 0x64, 0x00, 0xC8,
        ];
        let deserialized: otvar::ItemVariationStore = otspec::de::from_bytes(&binary_ivs).unwrap();
        let fivd = otvar::ItemVariationData {
            regionIndexes: vec![0],
            deltaValues: vec![vec![-200], vec![-50], vec![100], vec![200]],
        };
        let fivs = otvar::ItemVariationStore {
            format: 1,
            axisCount: 1,
            variationRegions: vec![vec![otvar::RegionAxisCoordinates {
                startCoord: 0.0,
                peakCoord: 1.0,
                endCoord: 1.0,
            }]],
            variationData: vec![fivd],
        };
        assert_eq!(deserialized, fivs);
    }
}
