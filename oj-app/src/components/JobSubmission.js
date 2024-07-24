import React, { useState } from 'react';
import axios from 'axios';
import './JobSubmission.css';
import 'bootstrap/dist/css/bootstrap.min.css';
const JobSubmission = () => {
    const [sourceCode, setSourceCode] = useState('');
    const [language, setLanguage] = useState('Rust');
    const [userId, setUserId] = useState('');
    const [contestId, setContestId] = useState('');
    const [problemId, setProblemId] = useState('');
    const [response, setResponse] = useState(null);
    const [requestText, setRequestText] = useState('');
    const [errorText, setErrorText] = useState('');
    const [jobId, setJobId] = useState('');
    const [delId, setdelId] = useState('');
    const handleSubmit = async (e) => {
        e.preventDefault();
        try {
            const requestBody = {
                source_code: sourceCode,
                language: language,
                user_id: parseInt(userId),
                contest_id: parseInt(contestId),
                problem_id: parseInt(problemId)
            };

            const requestString = `POST /jobs HTTP/1.1
Host: localhost:12345
Content-Type: application/json
Accept: application/json

${JSON.stringify(requestBody, null, 2)}`;

            setRequestText(requestString);

            console.log('Sending request:', requestString);

            const res = await axios.post('http://localhost:12345/jobs', requestBody, { timeout: 5000 });
            console.log('Received response:', res.data);
            setResponse(res.data);
            setErrorText('');
        } catch (error) {
            setResponse(null);
            let detailedError = "Error submitting job";
            if (error.response) {
                detailedError = `Status: ${error.response.status}\nData: ${JSON.stringify(error.response.data, null, 2)}`;
            } else if (error.request) {
                detailedError = "No response received from the server.";
            } else {
                detailedError = `Request error: ${error.message}`;
            }
            setErrorText(detailedError);
            console.error('Error submitting job:', error);
        }
    };
    const handlePut = async (e) => {
    e.preventDefault();
    const jobIdNumber = parseInt(jobId, 10);
    if (isNaN(jobIdNumber)) {
        setErrorText("Invalid Job ID. Please enter a valid numeric Job ID.");
        return;
    }

    try {
        const res = await axios.put(`http://localhost:12345/jobs/${jobIdNumber}`, null, { timeout: 5000 });
        console.log('Received response:', res.data);
        setResponse(res.data);
        setErrorText('');
    } catch (error) {
        setResponse(null);
        let detailedError = "Error updating job";
        if (error.response) {
            detailedError = `Status: ${error.response.status}\nData: ${JSON.stringify(error.response.data, null, 2)}`;
        } else if (error.request) {
            detailedError = "No response received from the server.";
        } else {
            detailedError = `Request error: ${error.message}`;
        }
        setErrorText(detailedError);
        console.error('Error updating job:', error);
    }
};
const handleDel = async (e) => {
    e.preventDefault();
    const jobIdNumber = parseInt(delId, 10);
    if (isNaN(jobIdNumber)) {
        setErrorText("Invalid Job ID. Please enter a valid numeric Job ID.");
        return;
    }

    try {
        const res = await axios.delete(`http://localhost:12345/jobs/${jobIdNumber}`, null, { timeout: 5000 });
        console.log('Received response:', res.status);
        setErrorText('');
    } catch (error) {
        setResponse(null);
        let detailedError = "Error updating job";
        if (error.response) {
            detailedError = `Status: ${error.response.status}\nData: ${JSON.stringify(error.response.data, null, 2)}`;
        } else if (error.request) {
            detailedError = "No response received from the server.";
        } else {
            detailedError = `Request error: ${error.message}`;
        }
        setErrorText(detailedError);
        console.error('Error updating job:', error);
    }
};


    return (
        <div className="job-submission-container">
            <h2 className='section-title'>Submit Job</h2>
            <form onSubmit={handleSubmit}>
                <textarea
                    value={sourceCode}
                    onChange={(e) => setSourceCode(e.target.value)}
                    rows="10"
                    cols="50"
                    placeholder="Enter your source code here"
                    className="input-field"
                />
                <br />
                <input
                    type="text"
                    placeholder="Enter language"
                    value={language}
                    onChange={(e) => setLanguage(e.target.value)}
                    className="input-field"
                />
                <br />
                <input
                    type="text"
                    placeholder="Enter user ID"
                    value={userId}
                    onChange={(e) => setUserId(e.target.value)}
                    className="input-field"
                />
                <br />
                <input
                    type="text"
                    placeholder="Enter contest ID"
                    value={contestId}
                    onChange={(e) => setContestId(e.target.value)}
                    className="input-field"
                />
                <br />
                <input
                    type="text"
                    placeholder="Enter problem ID"
                    value={problemId}
                    onChange={(e) => setProblemId(e.target.value)}
                    className="input-field"
                />
                <br />
                <button type="submit" className="btn btn-success">POST</button>

            </form>
            <form onSubmit={handlePut}>
                <br />
                <input
                    type="text"
                    placeholder="Enter Job ID"
                    value={jobId}
                    onChange={(e) => setJobId(e.target.value)}
                    className="input-field"
                />
                <br />
                <button type="submit" className="btn btn-warning">PUT</button>
            </form>
             <form onSubmit={handleDel}>
                <br />
                <input
                    type="text"
                    placeholder="Enter Job ID"
                    value={delId}
                    onChange={(e) => setdelId(e.target.value)}
                    className="input-field"
                />
                <br />
                <button type="submit" className="btn btn-danger">DELETE</button>
            </form>
            {requestText && (
                <div>
                    <h3>HTTP Request</h3>
                    <pre>{requestText}</pre>
                </div>
            )}
            {response && (
                <div className="response-container">
                    <h3>Job Submission Result</h3>
                    <p><strong>Result:</strong> {response.result}</p>
                    <p><strong>Score:</strong> {response.score}</p>
                    <h4>Details:</h4>
                    {response.cases.map((testCase) => (
                        <div key={testCase.id} className="case-result">
                            <p><strong>Case {testCase.id}:</strong> {testCase.result}</p>
                            <p><strong>Time:</strong> {testCase.time} us</p>
                            <p><strong>Memory:</strong> {testCase.memory} KB</p>
                        </div>
                    ))}
                </div>
            )}
            {errorText && (
                <div className="error-container">
                    <h3>HTTP Error</h3>
                    <pre>{errorText}</pre>
                </div>
            )}
        </div>
    );
};

export default JobSubmission;
