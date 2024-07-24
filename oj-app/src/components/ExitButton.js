// src/components/ExitButton.js
import React from 'react';

const ExitButton = () => {
    const handleExit = async () => {
        try {
            const response = await fetch('http://localhost:12345/internal/exit', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            });

            if (response.ok) {
                alert('Server is shutting down.');
            } else {
                alert(`Failed to shut down server: ${response.statusText}`);
            }
        } catch (error) {
            console.error('Error:', error);
            alert('An error occurred while attempting to shut down the server.');
        }
    };

    return (
        <button onClick={handleExit}>Shut Down Server</button>
    );
};

export default ExitButton;
