I have a big assignment for you.
Currently we are using a weigting system between the factors (sigma), I would recommend you to read the docs under magics/api/docs/ find the factor_graph.md and weights.md

The weighting system is not really RL friendly as it is not adding up to 1 or anything.


---

We have a pause functionality in the config:
pause-on-spawn
pause-on-load

However pause-on-load is pausing before the spawn is happening (i think), so the spawns that should happen at time 0.0 is not happening. 
