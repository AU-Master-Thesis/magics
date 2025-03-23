"""
Magics ZeroMQ Client

This module provides a client for communicating with the Magics simulation
using ZeroMQ.
"""

import time
import uuid
import json
import zmq
import numpy as np
from typing import Dict, List, Optional, Any, Tuple


class MagicsError(Exception):
    """Exception raised for errors from the Magics API."""
    pass


class MagicsClient:
    """
    Client for communicating with the Magics simulation using ZeroMQ.
    
    This client uses the ZeroMQ REQ-REP pattern to send commands to the
    simulation and receive responses.
    """
    
    def __init__(self, host: str = "localhost", port: int = 5555, timeout: int = 5000):
        """
        Initialize the Magics client.
        
        Args:
            host: Hostname or IP address of the Magics server
            port: Port number of the Magics server
            timeout: Timeout in milliseconds for receiving responses
        """
        self.context = zmq.Context()
        self.socket = self.context.socket(zmq.REQ)
        self.socket.setsockopt(zmq.RCVTIMEO, timeout)
        self.socket.connect(f"tcp://{host}:{port}")
    
    def _send_request(self, command: str, **parameters) -> Any:
        """
        Send a request to the server and return the response.
        
        Args:
            command: Command to execute
            **parameters: Command parameters
        
        Returns:
            The response data from the server
        
        Raises:
            MagicsError: If the server returns an error
            zmq.ZMQError: If there's a ZMQ error
        """
        # Create request with nested command structure
        request = {
            "command": {
                "command": command
            },
            "request_id": str(uuid.uuid4())
        }
        
        # Add parameters if provided
        if parameters:
            request["command"]["parameters"] = parameters
        
        # Convert request to JSON
        request_json = json.dumps(request)
        
        # Send request
        self.socket.send_string(request_json)
        
        # Receive response
        try:
            response_str = self.socket.recv_string()
            response = json.loads(response_str)
        except zmq.ZMQError as e:
            if e.errno == zmq.EAGAIN:
                raise MagicsError("Timeout while waiting for response")
            raise
        
        # Check for errors
        if response.get("status") != "Success":
            raise MagicsError(response.get("error", "Unknown error"))
        
        # Return data
        return response.get("data")
    
    def get_agent_state(self) -> Dict[str, Dict[str, Any]]:
        """
        Get the state of all agents in the simulation.
        
        Returns:
            Dictionary mapping agent IDs to agent states
        """
        data = self._send_request("GetAgentState")
        
        # Check data format
        if not data:
            raise MagicsError("No data returned from get_agent_state")
        
        if "type" not in data or data["type"] != "AgentStates":
            raise MagicsError(f"Invalid response format for get_agent_state: {data}")
        
        # Process agent states
        result = {}
        for agent_id, state in data["content"].items():
            # Convert positions and velocities to numpy arrays
            if "position" in state:
                state["position"] = np.array(state["position"], dtype=np.float32)
            if "velocity" in state:
                state["velocity"] = np.array(state["velocity"], dtype=np.float32)
            result[agent_id] = state
        
        return result
    
    def get_environment_state(self) -> Dict[str, Any]:
        """
        Get the state of the environment.
        
        Returns:
            Dictionary containing environment state
        """
        data = self._send_request("GetEnvironmentState")
        
        # Check data format
        if not data:
            raise MagicsError("No data returned from get_environment_state")
        
        if "type" not in data or data["type"] != "EnvironmentState":
            raise MagicsError(f"Invalid response format for get_environment_state: {data}")
        
        # Process environment state
        result = data["content"]
        
        # Convert obstacles to numpy arrays
        if "obstacles" in result:
            result["obstacles"] = np.array(result["obstacles"], dtype=np.float32)
        
        # Convert boundaries to numpy arrays
        if "boundaries" in result:
            result["boundaries"] = {
                "min": np.array(result["boundaries"][0], dtype=np.float32),
                "max": np.array(result["boundaries"][1], dtype=np.float32)
            }
        
        return result
    
    def set_factor_weights(self, weights: Dict[str, float], agent_id: Optional[int] = None) -> None:
        """
        Set factor graph weights.
        
        Args:
            weights: Dictionary mapping factor names to weights
            agent_id: Optional agent ID for per-agent weights
        """
        self._send_request("SetFactorWeights", weights=weights, agent_id=agent_id)
    
    def step(self) -> None:
        """
        Step the simulation forward by one frame.
        """
        self._send_request("Step")
    
    def reset(self) -> None:
        """
        Reset the simulation.
        """
        self._send_request("Reset")
    
    def is_api_active(self) -> bool:
        """
        Check if the API is active.
        
        Returns:
            True if the API is active, False otherwise
        """
        data = self._send_request("IsApiActive")
        
        # Check data format
        if not data:
            raise MagicsError("No data returned from is_api_active")
        
        if "type" not in data or data["type"] != "Boolean":
            raise MagicsError(f"Invalid response format for is_api_active: {data}")
        
        return data["content"]
    
    def set_api_active(self, active: bool) -> None:
        """
        Set the API active state.
        
        Args:
            active: Whether to activate the API
        """
        self._send_request("SetApiActive", active=active)
    
    def send_command(self, command: str, parameters: Dict[str, Any]) -> Any:
        """
        Send a custom command to the server.
        
        Args:
            command: Command name
            parameters: Command parameters
            
        Returns:
            The response data from the server
        """
        return self._send_request(command, **parameters)
    
    def set_iterations_per_step(self, iterations: int) -> None:
        """
        Set the number of iterations per step.
        
        Args:
            iterations: The number of iterations per step
        """
        self._send_request("SetIterationsPerStep", iterations=iterations)
    
    def get_simulation_hz(self) -> float:
        """
        Get the simulation Hz (frequency).
        
        Returns:
            The current simulation Hz
        """
        data = self._send_request("GetSimulationHz")
        
        # Check data format
        if not data:
            raise MagicsError("No data returned from get_simulation_hz")
        
        if "type" not in data or data["type"] != "Number":
            raise MagicsError(f"Invalid response format for get_simulation_hz: {data}")
        
        return data["content"]
    
    def set_simulation_hz(self, hz: float) -> None:
        """
        Set the simulation Hz (frequency).
        
        Args:
            hz: The new Hz value (must be greater than 0)
        """
        if hz <= 0:
            raise ValueError("Hz must be greater than 0")
        
        self._send_request("SetSimulationHz", hz=hz)
    
    def close(self) -> None:
        """
        Close the connection.
        """
        self.socket.close()
        self.context.term()
