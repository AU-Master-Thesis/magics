#!/usr/bin/env python3
"""
Debug script for ZeroMQ communication with Magics.

This script tests all API commands to verify the ZeroMQ communication with the Magics server.
"""

import json
import uuid
import zmq
import time
import numpy as np

def send_request(socket, command, parameters=None, request_id=None):
    """Send a request to the server and return the response."""
    if request_id is None:
        request_id = str(uuid.uuid4())
    
    # Create request with nested command structure
    request = {
        "command": {
            "command": command
        },
        "request_id": request_id
    }
    
    # Add parameters if provided
    if parameters:
        request["command"]["parameters"] = parameters
    
    # Send request
    request_json = json.dumps(request)
    print(f"Sending request: {request_json}")
    socket.send_string(request_json)
    
    # Receive response
    try:
        response_str = socket.recv_string()
        print(f"Raw response: {response_str}")
        
        # Parse response
        response = json.loads(response_str)
        print(f"Parsed response: {json.dumps(response, indent=2)}")
        
        return response
    except zmq.ZMQError as e:
        if e.errno == zmq.EAGAIN:
            print("❌ Timeout while waiting for response")
        else:
            print(f"❌ ZMQ error: {e}")
        return None

def test_is_api_active(socket):
    """Test the IsApiActive command."""
    print("\n=== Testing IsApiActive ===")
    response = send_request(socket, "IsApiActive")
    
    if response:
        if response.get("status") == "Success":
            print("✅ Command succeeded")
            
            data = response.get("data")
            if data:
                if isinstance(data, dict) and data.get("type") == "Boolean":
                    is_active = data.get("content")
                    print(f"API active: {is_active}")
                    print("✅ Response format is correct")
                    return is_active
                else:
                    print(f"❌ Unexpected data format: {data}")
            else:
                print("❌ Response is missing data")
        else:
            print(f"❌ Command failed: {response.get('error', 'Unknown error')}")
    
    return None

def test_set_api_active(socket, active=True):
    """Test the SetApiActive command."""
    print(f"\n=== Testing SetApiActive({active}) ===")
    response = send_request(socket, "SetApiActive", {"active": active})
    
    if response:
        if response.get("status") == "Success":
            print(f"✅ Successfully set API active to {active}")
            return True
        else:
            print(f"❌ Command failed: {response.get('error', 'Unknown error')}")
    
    return False

def test_get_agent_state(socket):
    """Test the GetAgentState command."""
    print("\n=== Testing GetAgentState ===")
    response = send_request(socket, "GetAgentState")
    
    if response:
        if response.get("status") == "Success":
            print("✅ Command succeeded")
            
            data = response.get("data")
            if data and data.get("type") == "AgentStates":
                agents = data.get("content", {})
                print(f"Received data for {len(agents)} agents")
                
                # Print some agent details if available
                for agent_id, state in list(agents.items())[:2]:  # Show first 2 agents
                    print(f"Agent {agent_id}:")
                    if "position" in state:
                        print(f"  Position: {state['position']}")
                    if "velocity" in state:
                        print(f"  Velocity: {state['velocity']}")
                
                return agents
            else:
                print(f"❌ Unexpected data format: {data}")
        else:
            print(f"❌ Command failed: {response.get('error', 'Unknown error')}")
    
    return None

def test_get_environment_state(socket):
    """Test the GetEnvironmentState command."""
    print("\n=== Testing GetEnvironmentState ===")
    response = send_request(socket, "GetEnvironmentState")
    
    if response:
        if response.get("status") == "Success":
            print("✅ Command succeeded")
            
            data = response.get("data")
            if data and data.get("type") == "EnvironmentState":
                env_state = data.get("content", {})
                obstacles = env_state.get("obstacles", [])
                boundaries = env_state.get("boundaries", [])
                
                print(f"Environment has {len(obstacles)} obstacles")
                if boundaries:
                    print(f"Boundaries: {boundaries}")
                
                return env_state
            else:
                print(f"❌ Unexpected data format: {data}")
        else:
            print(f"❌ Command failed: {response.get('error', 'Unknown error')}")
    
    return None

def test_set_factor_weights(socket):
    """Test the SetFactorWeights command."""
    print("\n=== Testing SetFactorWeights ===")
    
    # Define some test weights
    weights = {
        "dynamic": 1.0,
        "obstacle": 2.0,
        "interrobot": 1.5,
        "tracking": 1.0
    }
    
    response = send_request(socket, "SetFactorWeights", {"weights": weights})
    
    if response:
        if response.get("status") == "Success":
            print("✅ Successfully set factor weights")
            return True
        else:
            print(f"❌ Command failed: {response.get('error', 'Unknown error')}")
    
    return False

def test_step(socket):
    """Test the Step command."""
    print("\n=== Testing Step ===")
    response = send_request(socket, "Step")
    
    if response:
        if response.get("status") == "Success":
            print("✅ Successfully stepped simulation")
            return True
        else:
            print(f"❌ Command failed: {response.get('error', 'Unknown error')}")
    
    return False

def test_reset(socket):
    """Test the Reset command."""
    print("\n=== Testing Reset ===")
    response = send_request(socket, "Reset")
    
    if response:
        if response.get("status") == "Success":
            print("✅ Successfully reset simulation")
            return True
        else:
            print(f"❌ Command failed: {response.get('error', 'Unknown error')}")
    
    return False

def main():
    print("ZeroMQ Debug Client")
    print("===================")
    
    # Create ZMQ context and socket
    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    socket.setsockopt(zmq.RCVTIMEO, 5000)  # 5 second timeout
    
    # Connect to server
    server_address = "tcp://localhost:5555"
    print(f"Connecting to server at {server_address}...")
    socket.connect(server_address)
    
    try:
        # Test all API commands
        is_active = test_is_api_active(socket)
        
        if is_active is not None and not is_active:
            # If API is not active, activate it
            test_set_api_active(socket, True)
        
        # Test getting state information
        test_get_agent_state(socket)
        test_get_environment_state(socket)
        
        # Test modifying simulation
        test_set_factor_weights(socket)
        test_step(socket)
        
        # Test reset
        test_reset(socket)
        
        # Test deactivating API
        test_set_api_active(socket, False)
        
        # Test if API is now inactive
        test_is_api_active(socket)
    
    finally:
        # Clean up
        print("\nCleaning up...")
        socket.close()
        context.term()
        print("Done.")

if __name__ == "__main__":
    main()
