"use client";

import { useState } from "react";

const API_BASE = "https://photoencoder-production.up.railway.app";

export default function Home() {
  const [imageFile, setImageFile] = useState(null);
  const [secretText, setSecretText] = useState("");
  const [password, setPassword] = useState("");
  const [resultUrl, setResultUrl] = useState(null);
  const [loading, setLoading] = useState(false);
  const [mode, setMode] = useState("encode"); // "encode" or "decode"
  const [error, setError] = useState(null);

  async function handleSubmit(e) {
    e.preventDefault();
    setError(null);
    setLoading(true);
    setResultUrl(null);

    if (!imageFile) {
      setError("Please upload an image file");
      setLoading(false);
      return;
    }
    if (mode === "encode" && !secretText.trim()) {
      setError("Please enter secret text to encode");
      setLoading(false);
      return;
    }

    const formData = new FormData();
    formData.append("image", imageFile);

    if (mode === "encode") {
      // convert secret text to bytes and append as a Blob with filename and type
      const secretBlob = new Blob([secretText], { type: "text/plain" });
      formData.append("secret", secretBlob, "secret.txt");
    }

    if (password.trim()) {
      formData.append("password", password);
    }

    try {
      const endpoint = mode === "encode" ? "/encode" : "/decode";

      const res = await fetch(`${API_BASE}${endpoint}`, {
        method: "POST",
        body: formData,
      });

      if (!res.ok) {
        const text = await res.text();
        throw new Error(text || "Server error");
      }

      if (mode === "encode") {
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        setResultUrl(url);
      } else {
        const blob = await res.blob();
        const text = await blob.text();
        setResultUrl(null);
        alert("Decoded secret:\n" + text);
      }
    } catch (err) {
      setError(err.message || "Unknown error");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="flex flex-col items-center justify-center min-h-screen bg-black text-white p-6">
      <h1 className="text-8xl mt-[-15rem] mb-8 font-semibold">Photo <span className="bg-gradient-to-r from-pink-600 via-violet-700 to-purple-800 text-transparent bg-clip-text">Encoder</span></h1>
      <h2 className="text-xl mb-4">Select mode</h2>
      <div className="mb-6 flex space-x-4">
        <button
          onClick={() => setMode("encode")}
          disabled={mode === "encode"}
          className={`px-6 py-2 rounded ${
            mode === "encode"
              ? "bg-green-600"
              : "bg-gray-700 hover:bg-gray-800 cursor-pointer"
          }`}
        >
          Encode
        </button>
        <button
          onClick={() => setMode("decode")}
          disabled={mode === "decode"}
          className={`px-6 py-2 rounded ${
            mode === "decode"
              ? "bg-green-600"
              : "bg-gray-700 hover:bg-gray-800 cursor-pointer"
          }`}
        >
          Decode
        </button>
      </div>

      <form
        onSubmit={handleSubmit}
        className="flex flex-col gap-4 w-full max-w-sm mt-[4rem] "
      >
        <input
          type="file"
          accept="image/png, image/jpeg"
          onChange={(e) => setImageFile(e.target.files?.[0] ?? null)}
          className="file:cursor-pointer file:bg-gray-800 file:text-white file:px-4 file:py-2 rounded"
          required
        />

        {mode === "encode" && (
          <textarea
            placeholder="Secret text to encode"
            value={secretText}
            onChange={(e) => setSecretText(e.target.value)}
            rows={5}
            className="resize-none p-2 rounded text-black"
            required={mode === "encode"}
          />
        )}

        <input
          type="password"
          placeholder="Password (optional)"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoComplete="off"
          className="px-4 py-2 rounded text-black"
        />

        <button
          type="submit"
          disabled={loading}
          className={`py-2 rounded font-semibold ${
            loading
              ? "bg-gray-600 cursor-not-allowed"
              : mode === "encode"
              ? "bg-blue-600 hover:bg-blue-700"
              : "bg-green-600 hover:bg-green-700"
          }`}
        >
          {loading
            ? mode === "encode"
              ? "Encoding..."
              : "Decoding..."
            : mode === "encode"
            ? "Encode"
            : "Decode"}
        </button>
      </form>

      {error && (
        <p className="mt-6 text-red-500 whitespace-pre-wrap">{error}</p>
      )}

      {resultUrl && mode === "encode" && (
        <div className="mt-8 flex flex-col items-center">
          <p className="mb-4 text-lg font-medium">Encoded Image:</p>
          <img
            src={resultUrl}
            alt="Encoded Result"
            className="max-w-full max-h-80 border border-white rounded"
          />
          <a
            href={resultUrl}
            download="encoded.png"
            className="mt-3 text-blue-400 hover:underline"
          >
            Download Encoded Image
          </a>
        </div>
      )}
    </main>
  );
}
