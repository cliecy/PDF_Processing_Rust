import { ReactNode, useState } from 'react';
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

  const pages: Record<PageType, ReactNode> = {
    home: <Home onNavigate={setCurrentPage} />,
    merge: <MergePdf />,
    split: <SplitPdf />,
    delete: <DeletePages />,
    extract: <ExtractPages />,
    compress: <CompressPdf />,
    rotate: <RotatePdf />,
    'images-to-pdf': <ImagesToPdf />,
    'pdf-to-images': <PdfToImages />,
  };

  return (
    <div className="flex h-screen bg-[#0f0f12]">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <main className="flex-1 overflow-auto">
        <div className="p-8">
          {(Object.entries(pages) as [PageType, ReactNode][]).map(([page, content]) => (
            <section
              key={page}
              className={currentPage === page ? 'animate-fade-in' : 'hidden'}
              aria-hidden={currentPage !== page}
            >
              {content}
            </section>
          ))}
        </div>
      </main>
    </div>
  );
}

export default App;
