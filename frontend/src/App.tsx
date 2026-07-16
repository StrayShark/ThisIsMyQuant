import { HashRouter, Routes, Route, Navigate, useLocation } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppShell } from "@/components/AppShell";
import { BootstrapLoader } from "@/components/BootstrapLoader";
import { useMarketStatus } from "@/hooks/useMarketStatus";
import { useAppBootstrap } from "@/hooks/useAppBootstrap";
import { useAppearance } from "@/hooks/useAppearance";
import { useNotifications } from "@/hooks/useNotifications";
import { api } from "@/api/client";
import { OverviewPage } from "@/pages/OverviewPage";
import { ReportsPage } from "@/pages/ReportsPage";
import { ReportDetailPage } from "@/pages/ReportDetailPage";
import { ReportComparePage } from "@/pages/ReportComparePage";
import { SettingsPage } from "@/pages/SettingsPage";
import { StatusPage } from "@/pages/StatusPage";
import { LandingPage } from "@/pages/LandingPage";
import { SimulationPage } from "@/pages/SimulationPage";
import { TradingReviewPage } from "@/pages/TradingReviewPage";
import { MarketReplayPage } from "@/pages/MarketReplayPage";
import { LocalDatabasePage } from "@/pages/LocalDatabasePage";
import { MarketsPage } from "@/pages/MarketsPage";
import { AssetDetailPage } from "@/pages/AssetDetailPage";
import { WatchlistPage } from "@/pages/WatchlistPage";
import { EventsPage } from "@/pages/EventsPage";
import { AiPage } from "@/pages/AiPage";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 30_000,
    },
  },
});

function MainAppRoutes() {
  useMarketStatus();
  useAppBootstrap();
  useAppearance();
  useNotifications();

  return (
    <AppShell>
      <Routes>
        <Route path="/" element={<OverviewPage />} />
        <Route path="/workspace" element={<Navigate to="/markets" replace />} />
        <Route path="/stocks" element={<Navigate to="/markets" replace />} />
        <Route path="/markets" element={<MarketsPage />} />
        <Route path="/markets/futures/:symbol" element={<AssetDetailPage />} />
        <Route path="/markets/stocks/:symbol" element={<AssetDetailPage />} />
        <Route path="/watchlist" element={<WatchlistPage />} />
        <Route path="/symbols" element={<Navigate to="/markets" replace />} />
        <Route path="/symbols/:symbol" element={<Navigate to="/markets/futures/:symbol" replace />} />
        <Route path="/news" element={<Navigate to="/events" replace />} />
        <Route path="/calendar" element={<Navigate to="/events" replace />} />
        <Route path="/events" element={<EventsPage />} />
        <Route path="/copilot" element={<Navigate to="/ai" replace />} />
        <Route path="/ai" element={<AiPage />} />
        <Route path="/factors" element={<Navigate to="/ai" replace />} />
        <Route path="/anomalies" element={<Navigate to="/events" replace />} />
        <Route path="/reports" element={<ReportsPage />} />
        <Route path="/reports/compare" element={<ReportComparePage />} />
        <Route path="/reports/:id" element={<ReportDetailPage />} />
        <Route path="/status" element={<StatusPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/simulation" element={<SimulationPage />} />
        <Route path="/review" element={<TradingReviewPage />} />
        <Route path="/replay" element={<MarketReplayPage />} />
        <Route path="/database" element={<LocalDatabasePage />} />
      </Routes>
    </AppShell>
  );
}

function AppGate() {
  const location = useLocation();
  const onSetup = location.pathname === "/setup";

  const { data: setup, isLoading } = useQuery({
    queryKey: ["llm-setup"],
    queryFn: () => api.getLlmSetup(),
    staleTime: 30_000,
  });

  if (onSetup) {
    return <LandingPage />;
  }

  if (isLoading) {
    return <BootstrapLoader />;
  }

  if (setup?.setup_required) {
    return <Navigate to="/setup" replace />;
  }

  return <MainAppRoutes />;
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Routes>
          <Route path="/*" element={<AppGate />} />
        </Routes>
      </HashRouter>
    </QueryClientProvider>
  );
}
