use std::{fs, path::Path};

use anyhow::Result;
use nix::unistd::Pid;
use oci_spec::{LinuxCpu, LinuxResources};

use crate::cgroups::common::{self, CGROUP_PROCS};

use super::Controller;

const CGROUP_CPU_SHARES: &str = "cpu.shares";
const CGROUP_CPU_QUOTA: &str = "cpu.cfs_quota_us";
const CGROUP_CPU_PERIOD: &str = "cpu.cfs_period_us";
const CGROUP_CPU_RT_RUNTIME: &str = "cpu.rt_runtime_us";
const CGROUP_CPU_RT_PERIOD: &str = "cpu.rt_period_us";

pub struct Cpu {}

impl Controller for Cpu {
    fn apply(linux_resources: &LinuxResources, cgroup_root: &Path, pid: Pid) -> Result<()> {
        log::debug!("Apply Cpu cgroup config");
        fs::create_dir_all(cgroup_root)?;
        if let Some(cpu) = &linux_resources.cpu {
            Self::apply(cgroup_root, cpu)?;
        }

        common::write_cgroup_file(cgroup_root.join(CGROUP_PROCS), pid)?;
        Ok(())
    }
}

impl Cpu {
    fn apply(root_path: &Path, cpu: &LinuxCpu) -> Result<()> {
        if let Some(cpu_shares) = cpu.shares {
            if cpu_shares != 0 {
                common::write_cgroup_file(root_path.join(CGROUP_CPU_SHARES), cpu_shares)?;
            }
        }

        if let Some(cpu_period) = cpu.period {
            if cpu_period != 0 {
                common::write_cgroup_file(root_path.join(CGROUP_CPU_PERIOD), cpu_period)?;
            }
        }

        if let Some(cpu_quota) = cpu.quota {
            if cpu_quota != 0 {
                common::write_cgroup_file(root_path.join(CGROUP_CPU_QUOTA), cpu_quota)?;
            }
        }

        if let Some(rt_runtime) = cpu.realtime_runtime {
            if rt_runtime != 0 {
                common::write_cgroup_file(root_path.join(CGROUP_CPU_RT_RUNTIME), rt_runtime)?;
            }
        }

        if let Some(rt_period) = cpu.realtime_period {
            if rt_period != 0 {
                common::write_cgroup_file(root_path.join(CGROUP_CPU_RT_PERIOD), rt_period)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cgroups::test::{set_fixture, setup, LinuxCpuBuilder};
    use std::fs;

    #[test]
    fn test_set_shares() {
        // arrange
        let (tmp, shares) = setup("test_set_shares", CGROUP_CPU_SHARES);
        let _ = set_fixture(&tmp, CGROUP_CPU_SHARES, "")
            .unwrap_or_else(|_| panic!("set test fixture for {}", CGROUP_CPU_SHARES));
        let cpu = LinuxCpuBuilder::new().with_shares(2048).build();

        // act
        Cpu::apply(&tmp, &cpu).expect("apply cpu");

        // assert
        let content = fs::read_to_string(shares)
            .unwrap_or_else(|_| panic!("read {} file content", CGROUP_CPU_SHARES));
        assert_eq!(content, 2048.to_string());
    }

    #[test]
    fn test_set_quota() {
        // arrange
        const QUOTA: i64 = 200000;
        let (tmp, max) = setup("test_set_quota", CGROUP_CPU_QUOTA);
        let cpu = LinuxCpuBuilder::new().with_quota(QUOTA).build();

        // act
        Cpu::apply(&tmp, &cpu).expect("apply cpu");

        // assert
        let content = fs::read_to_string(max)
            .unwrap_or_else(|_| panic!("read {} file content", CGROUP_CPU_QUOTA));
        assert_eq!(content, QUOTA.to_string());
    }

    #[test]
    fn test_set_period() {
        // arrange
        const PERIOD: u64 = 100000;
        let (tmp, max) = setup("test_set_period", CGROUP_CPU_PERIOD);
        let cpu = LinuxCpuBuilder::new().with_period(PERIOD).build();

        // act
        Cpu::apply(&tmp, &cpu).expect("apply cpu");

        // assert
        let content = fs::read_to_string(max)
            .unwrap_or_else(|_| panic!("read {} file content", CGROUP_CPU_PERIOD));
        assert_eq!(content, PERIOD.to_string());
    }

    #[test]
    fn test_set_rt_runtime() {
        // arrange
        const RUNTIME: i64 = 100000;
        let (tmp, max) = setup("test_set_rt_runtime", CGROUP_CPU_RT_RUNTIME);
        let cpu = LinuxCpuBuilder::new()
            .with_realtime_runtime(RUNTIME)
            .build();

        // act
        Cpu::apply(&tmp, &cpu).expect("apply cpu");

        // assert
        let content = fs::read_to_string(max)
            .unwrap_or_else(|_| panic!("read {} file content", CGROUP_CPU_RT_RUNTIME));
        assert_eq!(content, RUNTIME.to_string());
    }

    #[test]
    fn test_set_rt_period() {
        // arrange
        const PERIOD: u64 = 100000;
        let (tmp, max) = setup("test_set_rt_period", CGROUP_CPU_RT_PERIOD);
        let cpu = LinuxCpuBuilder::new().with_realtime_period(PERIOD).build();

        // act
        Cpu::apply(&tmp, &cpu).expect("apply cpu");

        // assert
        let content = fs::read_to_string(max)
            .unwrap_or_else(|_| panic!("read {} file content", CGROUP_CPU_RT_PERIOD));
        assert_eq!(content, PERIOD.to_string());
    }
}
