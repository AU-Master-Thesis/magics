use bevy::prelude::*;
use colored::Colorize;

use crate::{
    factorgraph::factorgraph::FactorGraph,
    planner::{
        collisions::resources::{RobotEnvironmentCollisions, RobotRobotCollisions},
        spawner::RobotClickedOn,
    },
};

use super::{
    bundle::{Ball, RadioAntenna, Radius, RobotConnections},
    mission::Mission,
};
use gbp_config::formation::PlanningStrategy;

/// Event handler called whenever the mesh of a robot is clicked
/// Meant for debugging purposes
pub fn on_robot_clicked(
    mut evr_robot_clicked_on: EventReader<RobotClickedOn>,
    robots: Query<(
        Entity,
        &Transform,
        &FactorGraph,
        &RobotConnections,
        &Radius,
        &Ball,
        &RadioAntenna,
        &Mission,
        &PlanningStrategy,
    )>,
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
) {
    for RobotClickedOn(robot_id) in evr_robot_clicked_on.read() {
        let Ok((
            _,
            transform,
            factorgraph,
            robotstate,
            radius,
            ball,
            antenna,
            mission,
            planning_strategy,
        )) = robots.get(*robot_id)
        else {
            error!("robot_id {:?} does not exist", robot_id);
            continue;
        };

        println!("----- robot cliked on -----");
        println!("{}: {:?}", "robot".blue(), robot_id);
        println!("  {}: {}", "radius".magenta(), radius.0);
        println!("  {}:", "antenna".magenta());
        println!("    {}: {}", "radius".cyan(), antenna.radius);
        println!(
            "    {}: {}",
            "on".cyan(),
            if antenna.active {
                "true".green()
            } else {
                "false".red()
            }
        );
        println!("  {}:", "state".magenta());
        let (_, current_variable) = factorgraph
            .first_variable()
            .expect("factorgraph should have >= 2 variables");
        let [px, py] = current_variable.estimated_position();
        let [vx, vy] = current_variable.estimated_velocity();
        println!("    {}: [{:.4}, {:.4}]", "position".cyan(), px, py);
        println!("    {}: [{:.4}, {:.4}]", "velocity".cyan(), vx, vy);

        println!(
            "    {}: {:?}",
            "neighbours".cyan(),
            robotstate.robots_within_comms_range
        );
        println!(
            "    {}: {:?}",
            "connected".cyan(),
            robotstate.robots_connected_with
        );
        let node_counts = factorgraph.node_count();
        let edge_count = factorgraph.edge_count();
        println!("  {}:", "factorgraph".magenta());
        println!("    {}: {}", "edges".cyan(), edge_count);
        println!("    {}: {}", "nodes".cyan(), node_counts.total());

        // Print factor weights
        println!("    {}:", "factor_weights".cyan());
        println!("      {}: {}", "dynamic".yellow(), factorgraph.factor_weights().dynamic);
        println!("      {}: {}", "obstacle".yellow(), factorgraph.factor_weights().obstacle);
        println!("      {}: {}", "interrobot".yellow(), factorgraph.factor_weights().interrobot);
        println!("      {}: {}", "tracking".yellow(), factorgraph.factor_weights().tracking);
        println!(
            "      {}: {}",
            "variable".red(),
            node_counts.variables.to_string().red()
        );
        println!(
            "      {}:  {}",
            "factors".red(),
            node_counts.factors.to_string().blue()
        );
        let factor_counts = factorgraph.factor_count();
        println!(
            "        {}: {}",
            "obstacle".yellow(),
            factor_counts.obstacle
        );
        println!("        {}: {}", "dynamic".yellow(), factor_counts.dynamic);
        println!(
            "        {}: {}",
            "interrobot".yellow(),
            factor_counts.interrobot
        );
        println!("        {}: {}", "pose".yellow(), node_counts.variables); // bundled together
        println!(
            "        {}: {}",
            "tracking".yellow(),
            factor_counts.tracking
        );

        println!("  {}:", "messages".magenta());
        let messages_sent = factorgraph.messages_sent();
        let messages_received = factorgraph.messages_received();
        println!("    {}:", "sent".cyan());
        println!("      {}: {}", "internal".red(), messages_sent.internal);
        println!("      {}: {}", "external".red(), messages_sent.external);
        println!("    {}:", "received".cyan());
        println!("      {}: {}", "internal".red(), messages_received.internal);
        println!("      {}: {}", "external".red(), messages_received.external);
        println!("  {}:", "collisions".magenta());
        let robot_collisions = robot_robot_collisions.get(*robot_id).unwrap_or(0);
        println!("    {}: {}", "other-robots".cyan(), robot_collisions);
        let env_collisions = robot_environment_collisions.get(*robot_id).unwrap_or(0);
        println!("    {}: {}", "environment".cyan(), env_collisions);
        println!("  {}:", "aabb".magenta());
        let position =
            parry2d::na::Isometry2::translation(transform.translation.x, transform.translation.z);
        let aabb = ball.aabb(&position);
        println!(
            "    {}: [{:.4}, {:.4}]",
            "min".cyan(),
            aabb.mins.x,
            aabb.mins.y
        );
        println!(
            "    {}: [{:.4}, {:.4}]",
            "max".cyan(),
            aabb.maxs.x,
            aabb.maxs.y
        );
        println!("    {}: {:.4} m^2", "area".cyan(), aabb.volume());
        println!(
            "  {}: {:?}",
            "planning_strategy".magenta(),
            planning_strategy
        );
        println!("  {}:", "mission".magenta());
        println!("    {}:", "taskpoints".cyan());
        mission.taskpoints.iter().for_each(|wp| {
            println!("      - {}", wp);
        });
        println!("    {}:", "routes".cyan());
        mission.routes.iter().for_each(|route| {
            println!("      - {}: {}", "target_index".red(), route.target_index); // target_index is pub
            println!("      - {}:", "waypoints".red());
            route.waypoints().iter().for_each(|wp| { // Use the public waypoints() method
                println!("        - {}", wp);
            });
        });
        println!("    {}: {}s", "started_at".cyan(), mission.started_at());
        println!("    {}: {:?}", "finished_at".cyan(), mission.finished_at());
        println!("    {}: {:?}", "state".cyan(), mission.state);
        println!("    {}: {}", "active_route".cyan(), mission.active_route);
        println!(
            "    {}: {:?}",
            "finished_when_intersects".cyan(),
            mission.finished_when_intersects
        );
        println!(
            "    {}: {:?}",
            "taskpoint_reached_when_intersects".cyan(),
            mission.taskpoint_reached_when_intersects
        );
    }
}
