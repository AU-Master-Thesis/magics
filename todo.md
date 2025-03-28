- We need track SOME agent info for every fixed timestep. As if it completes it will despawn and we have no information on it anymore, which means it is impossible to validate the last "step" that we have given it. 
  - So we need to track collision, so if it despawns we know if it has collided during the last api "step" call.
  - We need to track completed, so we know why it despawn, or why we are not getting information from it.
  - What else do we need, to start training a RL model. We just have to remember that a step in the api is multiple simulation steps. So if something big is happening in between, we need to be able to rely this information (This is mostly related to when a robot is despawned, as this is when we lose information.)

 