// Classical Error Correction using Reed-Solomon codes
use reed_solomon_erasure::ReedSolomon;

pub struct ClassicalErrorCorrection {
    rs: ReedSolomon,
}

impl ClassicalErrorCorrection {
    pub fn new(data_shards: usize, parity_shards: usize) -> Self {
        let rs = ReedSolomon::new(data_shards, parity_shards).unwrap();
        Self { rs }
    }
    
    pub fn encode(&self, data: &mut [Vec<u8>]) -> Result<(), String> {
        self.rs.encode(data).map_err(|e| format!("Encoding error: {:?}", e))
    }
    
    pub fn reconstruct(&self, data: &mut [Vec<u8>]) -> Result<(), String> {
        self.rs.reconstruct(data).map_err(|e| format!("Reconstruction error: {:?}", e))
    }
}
