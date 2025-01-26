import React from "react";
import { BrowserRouter as Router, Route, Routes } from "react-router-dom";
import Home from "./pages/Home";
import RandomQuoteGenerator from "./pages/RandomQuoteGenerator";
import Navbar from "./components/NavBar";
import './assets/styles/App.css';

const App: React.FC = () => {
  return (
    <Router>
      <div className="App">
        <Navbar />
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/random-quote" element={<RandomQuoteGenerator />} />
        </Routes>
      </div>
    </Router>
  );
};

export default App;
