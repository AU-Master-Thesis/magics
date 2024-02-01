


pub trait Factor {}

#[derive(Debug)]
struct DefaultFactor;
#[derive(Debug)]
struct DynamicFactor;
#[derive(Debug)]
struct InterRobotFactor;
#[derive(Debug)]
struct ObstacleFactor;

impl Factor for DefaultFactor {}
impl Factor for DynamicFactor {}
impl Factor for InterRobotFactor {}
impl Factor for ObstacleFactor {}

#[derive(Debug)]
enum FactorType {
    Default(DefaultFactor),
    Dynamic(DynamicFactor),
    InterRobot(InterRobotFactor),
    Ocstacle(ObstacleFactor),
}
