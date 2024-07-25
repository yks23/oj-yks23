import React from 'react';
import { BrowserRouter as Router, Route, Routes, Link } from 'react-router-dom';
import JobSubmission from './components/JobSubmission';
import JobStatus from './components/JobStatus';
import Ranklist from './components/Ranklist';
import ExitButton from './components/ExitButton';
import UserManagement from './components/UserManagement';
import Footer from './components/Footer';
import './App.css';
import 'bootstrap/dist/css/bootstrap.min.css';
const App = () => {
    return (
        <body className='bg-dark'>
        <Router>
            <div className="app-container bg-dark">
                <h1 className="Main-title">OJ System</h1>
                <nav className="sidebar-container">
                    <div className='sidebar'>
                    <Link to="/submit-job" className="nav-link">Submit Job</Link>
                    <Link to="/job-status" className="nav-link">Check Job Status</Link>
                    <Link to="/ranklist" className="nav-link">View Contest Ranklist</Link>
                    <Link to="/user-management" className="nav-link">User Management</Link>
                    <Link to="/exit" className="nav-link">Backdoor</Link>
                    </div>
                </nav>
                <Routes>
                    <Route path="/submit-job" element={<JobSubmissionPage />} />
                    <Route path="/job-status" element={<JobStatusPage />} />
                    <Route path="/ranklist" element={<RanklistPage />} />
                    <Route path="/user-management" element={<UserManagement />} />
                    <Route path="/exit" element={<div className="right-section button-container ">
                <ExitButton /> 
            </div>} />
                    <Route path="/" element={<HomePage />} />
                </Routes>
            </div>
            
            <div > <Footer /></div>
        </Router>
        </body>
    );
};

const HomePage = () => (
    <div className="right-section">
        <h2 className="right-section-title">Welcome to KSOJ System</h2>
        <p>Please select an option from the navigation menu.</p>
    </div>
);

const JobSubmissionPage = () => (
    <div>
        <h2 className="right-section-title">Submit Job</h2>
        <div className="right-section">
            <JobSubmission />
        </div>
    </div>
);

const JobStatusPage = () => (
    <div>
        <h2 className="right-section-title">Check Job Status</h2>
    <div className="right-section">
        <JobStatus />
    </div>
    </div>
);

const RanklistPage = () => {
    const [contestId, setContestId] = React.useState('');

    const handleContestIdChange = (e) => {
        setContestId(e.target.value);
    };

    return (<div>
        <h2 className="right-section-title">View Contest Ranklist</h2>
        <div className="right-section">
            
            <input
                type="text"
                placeholder="Enter contest ID"
                value={contestId}
                onChange={handleContestIdChange}
                className="input-field"
            />
            {contestId && <Ranklist contestId={contestId} />}
        </div>
        </div>
    );
};

export default App;
