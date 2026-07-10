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
import { DashboardPage } from "@/pages/DashboardPage";
import { ReportsPage } from "@/pages/ReportsPage";
import { ReportDetailPage } from "@/pages/ReportDetailPage";
import { ReportComparePage } from "@/pages/ReportComparePage";
import { SymbolsPage } from "@/pages/SymbolsPage";
import { SymbolDetailPage } from "@/pages/SymbolDetailPage";
import { FactorCenterPage } from "@/pages/FactorCenterPage";
import { NewsDecisionPage } from "@/pages/NewsDecisionPage";
import { MacroCalendarPage } from "@/pages/MacroCalendarPage";
import { AnomalyCenterPage } from "@/pages/AnomalyCenterPage";
import { CopilotPage } from "@/pages/CopilotPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { StatusPage } from "@/pages/StatusPage";
import { LandingPage } from "@/pages/LandingPage";
import { SimulationPage } from "@/pages/SimulationPage";
import { TradingReviewPage } from "@/pages/TradingReviewPage";
import { MarketReplayPage } from "@/pages/MarketReplayPage";
import { LocalDatabasePage } from "@/pages/LocalDatabasePage";
import { AStockPage } from "@/pages/AStockPage";

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
        <Route path="/workspace" element={<DashboardPage />} />
        <Route path="/stocks" element={<AStockPage />} />
        <Route path="/factors" element={<FactorCenterPage />} />
        <Route path="/news" element={<NewsDecisionPage />} />
        <Route path="/calendar" element={<MacroCalendarPage />} />
        <Route path="/anomalies" element={<AnomalyCenterPage />} />
        <Route path="/copilot" element={<CopilotPage />} />
        <Route path="/reports" element={<ReportsPage />} />
        <Route path="/reports/compare" element={<ReportComparePage />} />
        <Route path="/reports/:id" element={<ReportDetailPage />} />
        <Route path="/symbols" element={<SymbolsPage />} />
        <Route path="/symbols/:symbol" element={<SymbolDetailPage />} />
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
