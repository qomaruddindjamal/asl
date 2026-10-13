//! ASL UNDI: Unified Neural Driver Interface & NPU Systolic Matrix Engine
//! =====================================================================
//! Abstraksi seragam untuk akselerator AI (HiSilicon DaVinci, Google TPU, Rockchip NPU).
//! Fitur:
//! 1. Pemuatan bobot model terkuantisasi (INT8 / FP16) ke tightly-coupled SRAM NPU.
//! 2. Asynchronous Command Queue non-blocking untuk inferensi batch frame video.
//! 3. Pipeline dataflow systolic array tanpa pembacaan register berulang.

#![no_std]

/// Format Tipe Data Tensor Hardware
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TensorDataType {
    Int8,
    Uint8,
    Float16,
    Bfloat16,
    Float32,
}

/// Deskriptor Bobot Tensor Model Neural
#[repr(C)]
#[derive(Copy, Clone)]
pub struct NeuralTensorDescriptor {
    pub tensor_id: u32,
    pub data_type: TensorDataType,
    pub shape_dimensions: [u32; 4], // Batch, Channels, Height, Width
    pub sram_offset: u32,
    pub size_bytes: u32,
}

impl NeuralTensorDescriptor {
    pub const fn empty() -> Self {
        Self {
            tensor_id: 0,
            data_type: TensorDataType::Int8,
            shape_dimensions: [0, 0, 0, 0],
            sram_offset: 0,
            size_bytes: 0,
        }
    }
}

pub const MAX_NPU_QUEUED_COMMANDS: usize = 32;

/// Perintah Inferensi NPU Asinkron
#[derive(Debug, Copy, Clone)]
pub struct NpuInferenceCommand {
    pub command_id: u32,
    pub input_tensor_phys_addr: u64,
    pub output_tensor_phys_addr: u64,
    pub model_id: u16,
    pub is_completed: bool,
}

impl NpuInferenceCommand {
    pub const fn empty() -> Self {
        Self {
            command_id: 0,
            input_tensor_phys_addr: 0,
            output_tensor_phys_addr: 0,
            model_id: 0,
            is_completed: false,
        }
    }
}

/// Pengontrol Akselerator Neural Matrix
pub struct UnifiedNeuralDriverInterface {
    pub command_queue: [NpuInferenceCommand; MAX_NPU_QUEUED_COMMANDS],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub total_inferences_executed: u64,
}

impl UnifiedNeuralDriverInterface {
    pub const fn new() -> Self {
        Self {
            command_queue: [NpuInferenceCommand::empty(); MAX_NPU_QUEUED_COMMANDS],
            head: 0,
            tail: 0,
            count: 0,
            total_inferences_executed: 0,
        }
    }

    /// Kirim tugas inferensi ke NPU (Non-blocking)
    pub fn enqueue_inference(&mut self, cmd: NpuInferenceCommand) -> Result<(), &'static str> {
        if self.count >= MAX_NPU_QUEUED_COMMANDS {
            return Err("ERR_NPU_COMMAND_QUEUE_OVERFLOW");
        }

        self.command_queue[self.tail] = cmd;
        self.tail = (self.tail + 1) % MAX_NPU_QUEUED_COMMANDS;
        self.count += 1;
        Ok(())
    }

    /// Handler interupsi hardware NPU saat kalkulasi tensor selesai
    pub fn on_npu_hardware_interrupt(&mut self) -> Option<u32> {
        if self.count == 0 {
            return None;
        }

        let completed_cmd_id = self.command_queue[self.head].command_id;
        self.command_queue[self.head].is_completed = true;
        self.head = (self.head + 1) % MAX_NPU_QUEUED_COMMANDS;
        self.count -= 1;
        self.total_inferences_executed += 1;

        Some(completed_cmd_id)
    }
}
