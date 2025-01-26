import React, { useEffect, useState } from "react";
import axios from "axios";
import "../assets/styles/randomquote.css";

const RandomQuoteGenerator: React.FC = () => {
  const [quote, setQuote] = useState<string>("");
  const [season, setSeason] = useState<number | null>(null);
  const [episode, setEpisode] = useState<number | null>(null);
  const [speaker, setSpeaker] = useState<string>("");
  const [imageUrl, setImageUrl] = useState<string>("");
  const [font, setFont] = useState<string>("");
  const [fontSize, setFontSize] = useState<number>(18);

  const fontList = [
    "Roboto", "Open Sans", "Lato", "Montserrat", "Merriweather", "Poppins",
    "Arial", "Helvetica", "Georgia", "Times New Roman", "Courier New", "Verdana", 
    "Comic Sans MS", "Impact", "Cursive", "Fantasy", "Monospace", "Serif", "Sans-serif"
  ];

  useEffect(() => {
    const fetchQuote = async () => {
      try {
        const response = await axios.get("http://127.0.0.1:8081/random-line");
        const { content, season_id, episode_id, speaker_id } = response.data;
        setQuote(content);
        setSeason(season_id);
        setEpisode(episode_id);
        setSpeaker(`Speaker #${speaker_id}`);
      } catch (error) {
        console.error("Error fetching quote", error);
      }
    };

    const fetchImage = () => {
      const randomImage = `https://picsum.photos/500`;
      setImageUrl(randomImage);
    };

    const fetchFont = () => {
      const randomFont = fontList[Math.floor(Math.random() * fontList.length)];
      setFont(randomFont);
    };

    const calculateFontSize = () => {
      const baseFontSize = Math.floor(Math.random() * 30) + 20; // Random font size between 20 and 50
      const adjustedFontSize = Math.min(baseFontSize, 500 / quote.length); // Adjust based on quote length
      setFontSize(adjustedFontSize);
    };

    fetchQuote();
    fetchImage();
    fetchFont();
    calculateFontSize();
  }, []);

  const downloadImage = () => {
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    const img = new Image();

    img.onload = () => {
      canvas.width = img.width;
      canvas.height = img.height;
      
      ctx?.drawImage(img, 0, 0);
      ctx!.globalAlpha = 1;

      let fontSize = Math.floor(Math.random() * 50) + 20; // Random font size between 20 and 70
      ctx!.font = `bold ${fontSize}px ${font}`;
      ctx!.fillStyle = "#FFFFFF";
      ctx!.strokeStyle = "#FFFFFF";
      ctx!.textAlign = "center";

      while (ctx!.measureText(quote).width > canvas.width - 40 && fontSize > 10) {
        fontSize -= 2;
        ctx!.font = `bold ${fontSize}px ${font}`;
      }

      ctx!.fillText(quote, img.width / 2, img.height / 2);

      const dataUrl = canvas.toDataURL("image/png");
      const a = document.createElement("a");
      a.href = dataUrl;
      a.download = "quote-image.png";
      a.click();
    };

    img.src = imageUrl;
  };

  return (
    <div className="quote-container" style={{ position: "relative", width: "500px", height: "500px" }}>
      <img src={imageUrl} alt="Random background" style={{ width: "100%", height: "100%", objectFit: "cover" }} />
      <div
        className="quote-text"
        style={{
          fontFamily: font,
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          fontSize: `${fontSize}px`,
          color: "white",
          textAlign: "center",
          padding: "10px",
          borderRadius: "5px",
          width: "90%",
          fontWeight: "bold",
        }}
      >
        <p>{quote}</p>
      </div>
      <div className="quote-info" style={{ textAlign: "center", marginTop: "10px" }}>
        <p style={{ color: "white", fontSize: "12px" }}>Season: {season}</p>
        <p style={{ color: "white", fontSize: "12px" }}>Episode: {episode}</p>
        <p style={{ color: "white", fontSize: "12px" }}>{speaker}</p>
      </div>
      <button onClick={downloadImage} style={{ position: "absolute", bottom: "10px", right: "10px" }}>
        Download Image
      </button>
    </div>
  );
};

export default RandomQuoteGenerator;