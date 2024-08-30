use ament_index::message::RosMsg;

fn main() -> eyre::Result<()> {
    let ament_dirs = ament_index::index::ament_dirs()?;
    println!("AMENT directories: {ament_dirs:#?}");

    let ros_msgs: Vec<_> = ament_index::message::get_wanted_messages()?
        .into_iter()
        .map(|msg| {
            let RosMsg {
                module,
                prefix,
                name,
            } = msg;
            format!("{module}/{prefix}/{name}")
        })
        .collect();
    println!("ROS messages: {ros_msgs:#?}");

    Ok(())
}
