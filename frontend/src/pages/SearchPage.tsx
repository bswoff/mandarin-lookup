import SearchBar from '../components/SearchBar';

export default function SearchPage() {
  return (
    <main style={{ maxWidth: 600, margin: '80px auto', padding: '0 16px', textAlign: 'left', fontSize: 20 }}>
      <h1 style={{ fontSize: 36 }}>Mandarin Lookup</h1>
      <SearchBar />
    </main>
  );
}
