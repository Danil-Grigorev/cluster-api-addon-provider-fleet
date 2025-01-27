use kube::runtime::predicates;
use kube::ResourceExt;

pub fn generation_with_deletion(obj: &impl ResourceExt) -> Option<u64> {
    match obj.meta().deletion_timestamp {
        Some(_) => Some(0),
        None => predicates::generation(obj),
    }
}
