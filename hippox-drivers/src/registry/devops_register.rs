//! DevOps drivers registration

use crate::{DriverCategory, DriverRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Devops;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "devops", feature = "all"))]
    {
        use crate::drivers::docker::*;
        use crate::drivers::github::*;
        use crate::drivers::k8s::*;

        map.insert("k8s_get_pods".to_string(), Arc::new(K8sGetPodsDriver));
        map.insert(
            "k8s_describe_pod".to_string(),
            Arc::new(K8sDescribePodDriver),
        );
        map.insert("k8s_get_logs".to_string(), Arc::new(K8sGetLogsDriver));
        map.insert("k8s_exec".to_string(), Arc::new(K8sExecDriver));
        map.insert(
            "k8s_get_deployments".to_string(),
            Arc::new(K8sGetDeploymentsDriver),
        );
        map.insert(
            "k8s_scale_deployment".to_string(),
            Arc::new(K8sScaleDeploymentDriver),
        );
        map.insert(
            "k8s_restart_deployment".to_string(),
            Arc::new(K8sRestartDeploymentDriver),
        );
        map.insert("k8s_get_nodes".to_string(), Arc::new(K8sGetNodesDriver));
        map.insert(
            "k8s_get_namespaces".to_string(),
            Arc::new(K8sGetNamespacesDriver),
        );
        map.insert("k8s_apply_yaml".to_string(), Arc::new(K8sApplyYamlDriver));
        map.insert(
            "k8s_delete_resource".to_string(),
            Arc::new(K8sDeleteResourceDriver),
        );
        map.insert("docker_ps".to_string(), Arc::new(DockerPsDriver));
        map.insert(
            "docker_start_stop".to_string(),
            Arc::new(DockerStartStopDriver),
        );
        map.insert("docker_logs".to_string(), Arc::new(DockerLogsDriver));
        map.insert("docker_inspect".to_string(), Arc::new(DockerInspectDriver));
        map.insert("docker_exec".to_string(), Arc::new(DockerExecDriver));
        map.insert("github_get_repo".to_string(), Arc::new(GithubGetRepo));
        map.insert(
            "github_create_issue".to_string(),
            Arc::new(GithubCreateIssue),
        );
        map.insert("github_list_issues".to_string(), Arc::new(GithubListIssues));
        map.insert("github_star_repo".to_string(), Arc::new(GithubStarRepo));
        map.insert(
            "github_search_repos".to_string(),
            Arc::new(GithubSearchRepos),
        );
        map.insert("github_get_user".to_string(), Arc::new(GithubGetUser));
        map.insert("github_list_prs".to_string(), Arc::new(GithubListPRs));
    }
}
