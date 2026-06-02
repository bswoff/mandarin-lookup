import { BrowserRouter, Routes, Route } from 'react-router-dom';
import SearchPage from './pages/SearchPage';
import WordPage from './pages/WordPage';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<SearchPage />} />
        <Route path="/word/:pinyin/:slug" element={<WordPage />} />
      </Routes>
    </BrowserRouter>
  );
}
