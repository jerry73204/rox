use ament_dir::RosMsg;

fn main() -> eyre::Result<()> {
    let ament_dirs: Vec<_> = ament_dir::ament_dirs()?.collect();
    println!("AMENT directories: {ament_dirs:#?}");

    let ros_msgs: Vec<_> = ament_dir::get_wanted_messages()?
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
