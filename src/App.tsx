import { useState } from 'react';
import Sidebar from './components/Sidebar';
import Home from './pages/Home';
import MergePdf from './pages/MergePdf';
import SplitPdf from './pages/SplitPdf';
import DeletePages from './pages/DeletePages';
import ExtractPages from './pages/ExtractPages';
import CompressPdf from './pages/CompressPdf';
import RotatePdf from './pages/RotatePdf';
import ImagesToPdf from './pages/ImagesToPdf';
import PdfToImages from './pages/PdfToImages';

export type PageType =
  | 'home'
  | 'merge'
  | 'split'
  | 'delete'
  | 'extract'
  | 'compress'
  | 'rotate'
  | 'images-to-pdf'
  | 'pdf-to-images';

function App() {
  const [currentPage, setCurrentPage] = useState<PageType>('home');

  const renderPage = () => {
    switch (currentPage) {
      case 'home':
        return <Home onNavigate={setCurrentPage} />;
      case 'merge':
        return <MergePdf />;
      case 'split':
        return <SplitPdf />;
      case 'delete':
        return <DeletePages />;
      case 'extract':
        return <ExtractPages />;
      case 'compress':
        return <CompressPdf />;
      case 'rotate':
        return <RotatePdf />;
      case 'images-to-pdf':
        return <ImagesToPdf />;
      case 'pdf-to-images':
        return <PdfToImages />;
      default:
        return <Home onNavigate={setCurrentPage} />;
    }
  };

  return (
    <div className="flex h-screen bg-[#0f0f12]">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <main className="flex-1 overflow-auto">
        <div className="p-8 animate-fade-in">{renderPage()}</div>
      </main>
    </div>
  );
}

export default App;
