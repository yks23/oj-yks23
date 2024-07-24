import React, { useState, useEffect } from 'react';
import axios from 'axios';
import './Ranklist.css';

const Ranklist = ({ contestId }) => {
    const [ranklist, setRanklist] = useState([]);
    const [errorText, setErrorText] = useState('');
    const [scoringRule, setScoringRule] = useState('highest');
    const [tieBreaker, setTieBreaker] = useState('submission_time');

    const fetchRanklist = async () => {
        try {
            const response = await axios.get(`http://localhost:12345/contests/${contestId}/ranklist`, {
                params: {
                    scoring_rule: scoringRule,
                    tie_breaker: tieBreaker
                },
                timeout: 5000
            });
            setRanklist(response.data);
            setErrorText('');
        } catch (error) {
            let detailedError = "Error fetching ranklist";
            if (error.response) {
                detailedError = `Status: ${error.response.status}\nData: ${JSON.stringify(error.response.data, null, 2)}`;
            } else if (error.request) {
                detailedError = "No response received from the server.";
            } else {
                detailedError = `Request error: ${error.message}`;
            }
            setErrorText(detailedError);
            console.error('Error fetching ranklist:', error);
        }
    };

    useEffect(() => {
        if (contestId) {
            fetchRanklist();
        }
    }, [contestId, scoringRule, tieBreaker]);

    const handleScoringRuleChange = (e) => {
        setScoringRule(e.target.value);
    };

    const handleTieBreakerChange = (e) => {
        setTieBreaker(e.target.value);
    };

    const handleSubmit = (e) => {
        e.preventDefault();
        fetchRanklist();
    };

    return (
        <div className="ranklist-container">
            <form onSubmit={handleSubmit} className="filters">
                <label>
                    Scoring Rule:
                    <select value={scoringRule} onChange={handleScoringRuleChange} className="input-field">
                        <option value="highest">Highest</option>
                        <option value="latest">Latest</option>
                    </select>
                </label>
                <label>
                    Tie Breaker:
                    <select value={tieBreaker} onChange={handleTieBreakerChange} className="input-field">
                        <option value="submission_time">Submission Time</option>
                        <option value="submission_count">Submission Count</option>
                        <option value="user_id">User ID</option>
                    </select>
                </label>
                <button type="submit" className="submit-button">Get Ranklist</button>
            </form>
            {errorText && (
                <div className="error-container">
                    <h3>HTTP Error</h3>
                    <pre>{errorText}</pre>
                </div>
            )}
            {ranklist.length > 0 && (
                <div className="ranklist">
                    <h3>Ranklist</h3>
                    <table>
                        <thead>
                            <tr>
                                <th>Rank</th>
                                <th>User ID</th>
                                <th>User Name</th>
                                <th>Scores</th>
                            </tr>
                        </thead>
                        <tbody>
                            {ranklist.map((entry) => (
                                <tr key={entry.user.id}>
                                    <td>{entry.rank}</td>
                                    <td>{entry.user.id}</td>
                                    <td>{entry.user.name}</td>
                                    <td>{entry.scores.join(', ')}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
};

export default Ranklist;
