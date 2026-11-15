// fix.py: sysmem v1
//! RAM do sistema detectada automaticamente (Linux, macOS, Windows, FreeBSD...).
//!
//! Toda a dependência de `sysinfo` fica neste arquivo: se a API mudar numa versão
//! futura (ela já mudou: em 0.40 `System::new()` retorna `Result`), só aqui se ajusta.

use sysinfo::System;

use crate::spill::GIB;

/// Usado quando a RAM não pode ser detectada (conservador).
const RAM_SEM_DETECCAO: u64 = 16 * GIB;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RamDoSistema {
    /// RAM total (limitada pelo cgroup, se houver), em bytes.
    pub total: u64,
    /// RAM disponível agora (inclui cache reclamável), em bytes.
    pub disponivel: u64,
}

/// Lê a RAM do sistema. Em Linux dentro de contêiner (cgroup), respeita o limite do cgroup.
pub fn ram_do_sistema() -> Option<RamDoSistema> {
    let mut sistema = System::new().ok()?;
    sistema.refresh_memory();

    let mut total = sistema.total_memory();
    let mut disponivel = sistema.available_memory();

    // `cgroup_limits` só existe em Linux (retorna None nos demais sistemas).
    if let Some(cgroup) = sistema.cgroup_limits()
        && cgroup.total_memory > 0
    {
        total = total.min(cgroup.total_memory);
        disponivel = disponivel.min(cgroup.free_memory);
    }

    disponivel = disponivel.min(total);
    (total > 0).then_some(RamDoSistema { total, disponivel })
}

/// Orçamento de RAM disponível detectada pelo sistema, em bytes
/// (ou 16 GiB conservadores se a detecção falhar).
pub fn orcamento_de_ram() -> u64 {
    match ram_do_sistema() {
        Some(ram) => ram.disponivel.max(GIB),
        None => {
            log::warn!(
                "Não foi possível detectar a RAM do sistema: usando {} GiB.",
                RAM_SEM_DETECCAO / GIB
            );
            RAM_SEM_DETECCAO
        }
    }
}

#[cfg(test)]
mod tests_sysmem {
    use super::*;

    #[test]
    fn orcamento_automatico_tem_ao_menos_1_gib() {
        assert!(orcamento_de_ram() >= GIB);
    }

    #[test]
    fn ram_do_sistema_e_coerente() {
        if let Some(ram) = ram_do_sistema() {
            assert!(ram.total > 0);
            assert!(ram.disponivel <= ram.total);
        }
    }
}
