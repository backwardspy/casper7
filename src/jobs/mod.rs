use color_eyre::Result;
use tokio_cron_scheduler::Job;
use tracing::{error, info};

use crate::bot::JobContext;

mod meatball;

pub(crate) fn all(ctx: JobContext) -> Result<Vec<Job>> {
    let ctx = ctx.clone();
    Ok(vec![Job::new_async(
        "*/10 * * * * *",
        move |_uuid, _lock| {
            let ctx = ctx.clone();
            Box::pin(async move {
                match meatball::update_role_assignments(ctx).await {
                    Ok(()) => {
                        info!("Job meatball::update_role_assignments completed successfully.")
                    }
                    Err(e) => error!("Job meatball::update_role_assignments failed: {e}"),
                }
            })
        },
    )?])
}
