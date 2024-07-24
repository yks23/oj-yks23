import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './UserManagement.css';
import 'bootstrap/dist/css/bootstrap.min.css';
const UserManagement = () => {
    const [users, setUsers] = useState([]);
    const [name, setName] = useState('');
    const [id, setId] = useState('');

    useEffect(() => {
        fetchUsers();
    }, []);

    const fetchUsers = async () => {
        try {
            const response = await axios.get('http://localhost:12345/users');
            setUsers(response.data);
        } catch (error) {
            console.error('Error fetching users:', error);
        }
    };

    const registerUser = async () => {
        try {
            const user = id ? { id: parseInt(id, 10), name } : { name };
            const response = await axios.post('http://localhost:12345/users', user);
            alert(`User registered: ${JSON.stringify(response.data)}`);
            fetchUsers();
        } catch (error) {
            console.error('Error registering user:', error);
        }
    };

    return (
        <div className="user-management">
            <h2 className="section-title">User Management</h2>
            <div className="form-group">
                <input
                    type="text"
                    placeholder="Enter name"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="input-field"
                />
                <input
                    type="text"
                    placeholder="Enter ID (optional)"
                    value={id}
                    onChange={(e) => setId(e.target.value)}
                    className="input-field"
                />
                <button onClick={registerUser} className="btn btn-success">Register User</button>
            </div>
            <h3 className="section-title">Registered Users</h3>
            <ul className="user-list">
                {users.map((user) => (
                    <li key={user.id} className="user-item">
                        {`ID: ${user.id}, Name: ${user.name}`}
                    </li>
                ))}
            </ul>
        </div>
    );
};

export default UserManagement;
