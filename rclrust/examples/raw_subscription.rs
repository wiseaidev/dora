use anyhow::Result;
use rclrust::{qos::QoSProfile, rclrust_info};
use rclrust_msg::{_core::FFIToRust, std_msgs::msg::String as String_};

fn main() -> Result<()> {
    let ctx = rclrust::init()?;
    let node = ctx.create_node("examples_subscriber")?;
    let logger = node.logger();

    let _subscription = node.create_raw_subscription::<String_, _>(
        "message",
        move |msg| {
            rclrust_info!(logger, "{}", unsafe { msg.to_rust() }.data);
        },
        &QoSProfile::default(),
    )?;

    rclrust::spin(&node)?;

    Ok(())
}
