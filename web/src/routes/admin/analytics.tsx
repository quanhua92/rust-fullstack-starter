import { createFileRoute } from '@tanstack/react-router'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { TaskAnalytics } from '@/components/admin/TaskAnalytics'
import { HealthTrends } from '@/components/admin/HealthTrends'
import { UserActivityAnalytics } from '@/components/admin/UserActivityAnalytics'
import { RealTimeNotifications } from '@/components/admin/RealTimeNotifications'
import { 
  BarChart3, 
  TrendingUp, 
  Users, 
  Bell 
} from 'lucide-react'

export const Route = createFileRoute('/admin/analytics')({
  component: AnalyticsDashboard,
})

function AnalyticsDashboard() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">Analytics & Insights</h2>
        <p className="text-muted-foreground">
          Advanced data visualization and real-time monitoring for tasks, health, and user activity
        </p>
      </div>

      <Tabs defaultValue="tasks" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="tasks" className="flex items-center space-x-2">
            <BarChart3 className="h-4 w-4" />
            <span>Task Analytics</span>
          </TabsTrigger>
          <TabsTrigger value="health" className="flex items-center space-x-2">
            <TrendingUp className="h-4 w-4" />
            <span>Health Trends</span>
          </TabsTrigger>
          <TabsTrigger value="users" className="flex items-center space-x-2">
            <Users className="h-4 w-4" />
            <span>User Activity</span>
          </TabsTrigger>
          <TabsTrigger value="notifications" className="flex items-center space-x-2">
            <Bell className="h-4 w-4" />
            <span>Notifications</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="tasks" className="space-y-6">
          <TaskAnalytics />
        </TabsContent>

        <TabsContent value="health" className="space-y-6">
          <HealthTrends />
        </TabsContent>

        <TabsContent value="users" className="space-y-6">
          <UserActivityAnalytics />
        </TabsContent>

        <TabsContent value="notifications" className="space-y-6">
          <RealTimeNotifications />
        </TabsContent>
      </Tabs>
    </div>
  )
}