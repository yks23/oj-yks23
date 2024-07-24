// ModalImage.js
import React from 'react';
import './ModalImage.css';

const ModalImage = ({ imageUrl, setShowImage }) => {
    return (
        <div className="modal">
            <div className="modal-content">
                <span className="close" onClick={() => setShowImage(false)}>&times;</span>
                <img src={imageUrl} alt="弹出图片" className="modal-image" />
            </div>
        </div>
    );
};

export default ModalImage;
