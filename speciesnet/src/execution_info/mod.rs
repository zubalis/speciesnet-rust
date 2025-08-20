#[derive(Clone, Debug, Default)]
pub struct ExecutionInfo {
    coreml_enabled: bool,
    cuda_enabled: bool,
}

impl ExecutionInfo {
    pub fn new(coreml_enabled: bool, cuda_enabled: bool) -> Self {
        Self {
            coreml_enabled,
            cuda_enabled,
        }
    }

    pub fn is_gpu_enabled(&self) -> bool {
        self.coreml_enabled || self.cuda_enabled
    }
}
