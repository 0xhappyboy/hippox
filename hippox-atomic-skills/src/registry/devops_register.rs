//! DevOps skills registration

use crate::{SkillCategory, SkillRegistryMap};
use std::collections::HashMap;
use std::sync::Arc;

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Devops;
    let map = registry.entry(category).or_insert_with(HashMap::new);

    #[cfg(any(feature = "devops", feature = "all"))]
    {
        use crate::skills::docker::*;
        use crate::skills::github::*;
        use crate::skills::k8s::*;

        map.insert("k8s_get_pods".to_string(), Arc::new(K8sGetPodsSkill));
        map.insert(
            "k8s_describe_pod".to_string(),
            Arc::new(K8sDescribePodSkill),
        );
        map.insert("k8s_get_logs".to_string(), Arc::new(K8sGetLogsSkill));
        map.insert("k8s_exec".to_string(), Arc::new(K8sExecSkill));
        map.insert(
            "k8s_get_deployments".to_string(),
            Arc::new(K8sGetDeploymentsSkill),
        );
        map.insert(
            "k8s_scale_deployment".to_string(),
            Arc::new(K8sScaleDeploymentSkill),
        );
        map.insert(
            "k8s_restart_deployment".to_string(),
            Arc::new(K8sRestartDeploymentSkill),
        );
        map.insert("k8s_get_nodes".to_string(), Arc::new(K8sGetNodesSkill));
        map.insert(
            "k8s_get_namespaces".to_string(),
            Arc::new(K8sGetNamespacesSkill),
        );
        map.insert("k8s_apply_yaml".to_string(), Arc::new(K8sApplyYamlSkill));
        map.insert(
            "k8s_delete_resource".to_string(),
            Arc::new(K8sDeleteResourceSkill),
        );
        map.insert("docker_ps".to_string(), Arc::new(DockerPsSkill));
        map.insert(
            "docker_start_stop".to_string(),
            Arc::new(DockerStartStopSkill),
        );
        map.insert("docker_logs".to_string(), Arc::new(DockerLogsSkill));
        map.insert("docker_inspect".to_string(), Arc::new(DockerInspectSkill));
        map.insert("docker_exec".to_string(), Arc::new(DockerExecSkill));
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
