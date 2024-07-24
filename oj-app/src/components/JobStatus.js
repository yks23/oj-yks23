import React, { useState } from 'react';
import axios from 'axios';
import './JobStatus.css';
import ModalImage from './ModalImage';
import failedjpg from './th.jpg'
import successjpg from './success.jpg'
import 'bootstrap/dist/css/bootstrap.min.css';
const JobStatus = () => {
    const [jobId, setJobId] = useState('');
    const [jobStatus, setJobStatus] = useState(null);
    const [filter, setFilter] = useState({
        user_id: '',
        user_name: '',
        contest_id: '',
        problem_id: '',
        language: '',
        from: '',
        to: '',
        state: '',
        result: ''
    });
    const [jobList, setJobList] = useState([]);
    const [showImage, setShowImage] = useState(false);
    const [showfImage, setShowfImage] = useState(false);
    const fetchJobStatus = async () => {
        try {
            const response = await axios.get(`http://localhost:12345/jobs/${jobId}`);
            setJobStatus(response.data);

            if (response.data.result === 'Accepted') {
                setShowImage(true);
                setShowfImage(false);
            } else {
                setShowImage(false);
                setShowfImage(true);
            }
        } catch (error) {
            console.error('Error fetching job status:', error);
        }
    };

    const fetchJobList = async () => {
        try {
            const queryParams = new URLSearchParams();
            Object.keys(filter).forEach(key => {
                if (filter[key]) {
                    queryParams.append(key, filter[key]);
                }
            });
            const url = `http://localhost:12345/jobs?${queryParams.toString()}`;
            const response = await axios.get(url);
            setJobList(response.data);
        } catch (error) {
            console.error('Error fetching job list:', error);
        }
    };

    const handleInputChange = (e) => {
        const { name, value } = e.target;
        setFilter({
            ...filter,
            [name]: value
        });
    };

    const handleSubmit = (e) => {
        e.preventDefault();
        fetchJobList();
    };

    return (
        <div className="job-status">
            <h2 className="section-title">Check Job Status</h2>
            <div className="form-group">
                <input
                    type="text"
                    placeholder="Enter job ID"
                    value={jobId}
                    onChange={(e) => setJobId(e.target.value)}
                    className="input-field"
                />
                <button onClick={fetchJobStatus} className="btn btn-success">Check Status</button>
            </div>
            {jobStatus && (
                <div className="job-details">
                    <h3>Job Details</h3>
                    <div className="response-container">
                        <p><strong>Result:</strong> {jobStatus.result}</p>
                        <p><strong>Score:</strong> {jobStatus.score}</p>
                        <h4>Details:</h4>
                        {jobStatus.cases.map((testCase) => (
                            <div key={testCase.id} className="case-result">
                                <p><strong>Case {testCase.id}:</strong> {testCase.result}</p>
                                <p><strong>Time:</strong> {testCase.time} us</p>
                                <p><strong>Memory:</strong> {testCase.memory} KB</p>
                            </div>
                        ))}
                    </div>
                    {showfImage && (
                        <ModalImage imageUrl={failedjpg} setShowImage={setShowfImage} />
                    )
                    }
                    {showImage && (
                        <ModalImage imageUrl={successjpg} setShowImage={setShowImage} />
                    )
                    }
                </div>

            )}
            <h2 className="section-title">Filter Jobs</h2>
            <form onSubmit={handleSubmit} className="form-group">
                <input
                    type="text"
                    name="user_id"
                    placeholder="User ID"
                    value={filter.user_id}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="user_name"
                    placeholder="User Name"
                    value={filter.user_name}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="contest_id"
                    placeholder="Contest ID"
                    value={filter.contest_id}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="problem_id"
                    placeholder="Problem ID"
                    value={filter.problem_id}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="language"
                    placeholder="Language"
                    value={filter.language}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="from"
                    placeholder="From (UTC format)"
                    value={filter.from}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="to"
                    placeholder="To (UTC format)"
                    value={filter.to}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="state"
                    placeholder="State"
                    value={filter.state}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <input
                    type="text"
                    name="result"
                    placeholder="Result"
                    value={filter.result}
                    onChange={handleInputChange}
                    className="input-field"
                />
                <button type="submit" className="bt btn btn-success">Filter Jobs</button>
            </form>
            {jobList.length > 0 && (
                <div className="job-list">
                    <h3>Job List</h3>
                    {jobList.map((job) => (
                        <div key={job.id} className="job-item">
                            <div className="response-container">
                                <p><strong>Job ID:</strong> {job.id}</p>
                                <p><strong>Result:</strong> {job.result}</p>
                                <p><strong>Score:</strong> {job.score}</p>
                                <h4>Details:</h4>
                                {job.cases.map((testCase) => (
                                    <div key={testCase.id} className="case-result">
                                        <p><strong>Case {testCase.id}:</strong> {testCase.result}</p>
                                        <p><strong>Time:</strong> {testCase.time} us</p>
                                        <p><strong>Memory:</strong> {testCase.memory} KB</p>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default JobStatus;
