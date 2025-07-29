import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { apiClient } from '@/lib/api/client'
import { formatDistanceToNow } from 'date-fns'
import { 
  Server, 
  Database, 
  Clock, 
  Activity, 
  TrendingUp, 
  Zap,
  CheckCircle2,
  AlertTriangle,
  XCircle
} from 'lucide-react'

import type { components } from '@/types/api'

type ComponentHealth = components['schemas']['ComponentHealth']

export function SystemMetrics() {
  const basicHealthQuery = useQuery({
    queryKey: ['health', 'basic'],
    queryFn: () => apiClient.getHealth(),
    refetchInterval: 30000,
  })

  const detailedHealthQuery = useQuery({
    queryKey: ['health', 'detailed'],
    queryFn: () => apiClient.getDetailedHealth(),
    refetchInterval: 30000,
  })

  const taskStatsQuery = useQuery({
    queryKey: ['tasks', 'stats'],
    queryFn: () => apiClient.getTaskStats(),
    refetchInterval: 15000,
  })

  const formatUptime = (uptime: number) => {
    const uptimeMs = uptime * 1000
    const now = new Date()
    const startTime = new Date(now.getTime() - uptimeMs)
    return formatDistanceToNow(startTime, { addSuffix: true })
  }

  const getUptimePercentage = (uptime: number) => {
    // Assume 99.9% uptime target for visualization
    const targetUptime = 24 * 60 * 60 // 24 hours in seconds
    const currentUptime = Math.min(uptime, targetUptime)
    return Math.round((currentUptime / targetUptime) * 100)
  }

  const getHealthScore = () => {
    if (!detailedHealthQuery.data?.data?.checks) return 0
    
    const checks = Object.values(detailedHealthQuery.data.data.checks)
    const healthyChecks = checks.filter((check: ComponentHealth) => check.status === 'healthy').length
    return Math.round((healthyChecks / checks.length) * 100)
  }

  const getTaskCompletionRate = () => {
    if (!taskStatsQuery.data?.data) return 0
    
    const stats = taskStatsQuery.data?.data
    const totalProcessed = (stats.completed || 0) + (stats.failed || 0)
    if (totalProcessed === 0) return 100
    
    return Math.round(((stats.completed || 0) / totalProcessed) * 100)
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {/* System Uptime */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">System Uptime</CardTitle>
          <Clock className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {basicHealthQuery.data?.data?.uptime ? 
              formatUptime(basicHealthQuery.data.data.uptime) : 
              'Loading...'}
          </div>
          <div className="mt-2">
            <Progress 
              value={basicHealthQuery.data?.data?.uptime ? getUptimePercentage(basicHealthQuery.data.data.uptime) : 0} 
              className="h-2" 
            />
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            Since last restart
          </p>
        </CardContent>
      </Card>

      {/* Health Score */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Health Score</CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {getHealthScore()}%
          </div>
          <div className="mt-2">
            <Progress value={getHealthScore()} className="h-2" />
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            System dependencies healthy
          </p>
        </CardContent>
      </Card>

      {/* Task Performance */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Task Success Rate</CardTitle>
          <TrendingUp className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {getTaskCompletionRate()}%
          </div>
          <div className="mt-2">
            <Progress value={getTaskCompletionRate()} className="h-2" />
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            Tasks completed successfully
          </p>
        </CardContent>
      </Card>

      {/* Database Status */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Database</CardTitle>
          <Database className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="flex items-center space-x-2">
            {detailedHealthQuery.data?.data?.checks?.database?.status === 'healthy' ? (
              <CheckCircle2 className="h-5 w-5 text-green-600" />
            ) : (
              <XCircle className="h-5 w-5 text-red-600" />
            )}
            <div className="text-2xl font-bold">
              {detailedHealthQuery.data?.data?.checks?.database?.status || 'Unknown'}
            </div>
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            {detailedHealthQuery.data?.data?.checks?.database?.message || 'Checking connection...'}
          </p>
        </CardContent>
      </Card>

      {/* API Performance */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">API Status</CardTitle>
          <Server className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="flex items-center space-x-2">
            <CheckCircle2 className="h-5 w-5 text-green-600" />
            <div className="text-2xl font-bold">
              {basicHealthQuery.data?.data?.status || 'Unknown'}
            </div>
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            API v{basicHealthQuery.data?.data?.version || 'Unknown'}
          </p>
        </CardContent>
      </Card>

      {/* Active Tasks */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Active Tasks</CardTitle>
          <Zap className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {taskStatsQuery.data ? 
              ((taskStatsQuery.data?.data?.pending || 0) + (taskStatsQuery.data?.data?.running || 0)) : 
              'Loading...'}
          </div>
          <div className="flex space-x-2 mt-2">
            <Badge variant="outline" className="text-xs">
              {taskStatsQuery.data?.data?.pending || 0} pending
            </Badge>
            <Badge variant="outline" className="text-xs">
              {taskStatsQuery.data?.data?.running || 0} running
            </Badge>
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            Currently processing
          </p>
        </CardContent>
      </Card>

      {/* Detailed Task Breakdown */}
      <Card className="md:col-span-2 lg:col-span-3">
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Activity className="h-5 w-5" />
            <span>Task Statistics</span>
          </CardTitle>
          <CardDescription>
            Comprehensive breakdown of task processing metrics
          </CardDescription>
        </CardHeader>
        <CardContent>
          {taskStatsQuery.isLoading ? (
            <div>Loading task statistics...</div>
          ) : taskStatsQuery.data ? (
            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-600">
                  {taskStatsQuery.data?.data?.total || 0}
                </div>
                <div className="text-sm text-muted-foreground">Total</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-yellow-600">
                  {taskStatsQuery.data?.data?.pending || 0}
                </div>
                <div className="text-sm text-muted-foreground">Pending</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-purple-600">
                  {taskStatsQuery.data?.data?.running || 0}
                </div>
                <div className="text-sm text-muted-foreground">Running</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">
                  {taskStatsQuery.data?.data?.completed || 0}
                </div>
                <div className="text-sm text-muted-foreground">Completed</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-red-600">
                  {(taskStatsQuery.data?.data?.failed || 0) + (taskStatsQuery.data?.data?.cancelled || 0)}
                </div>
                <div className="text-sm text-muted-foreground">
                  Failed + Cancelled
                </div>
              </div>
            </div>
          ) : (
            <div className="text-center text-muted-foreground">
              Failed to load task statistics
            </div>
          )}
        </CardContent>
      </Card>

      {/* System Dependencies */}
      <Card className="md:col-span-2 lg:col-span-3">
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Database className="h-5 w-5" />
            <span>System Dependencies</span>
          </CardTitle>
          <CardDescription>
            Current status of critical system dependencies
          </CardDescription>
        </CardHeader>
        <CardContent>
          {detailedHealthQuery.isLoading ? (
            <div>Loading dependency status...</div>
          ) : detailedHealthQuery.data?.data?.checks ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {Object.entries(detailedHealthQuery.data.data.checks).map(([name, check]: [string, ComponentHealth]) => (
                <div key={name} className="flex items-center justify-between p-3 border rounded-lg">
                  <div className="flex items-center space-x-3">
                    {check.status === 'healthy' ? (
                      <CheckCircle2 className="h-5 w-5 text-green-600" />
                    ) : check.status === 'degraded' ? (
                      <AlertTriangle className="h-5 w-5 text-yellow-600" />
                    ) : (
                      <XCircle className="h-5 w-5 text-red-600" />
                    )}
                    <div>
                      <div className="font-medium capitalize">{name}</div>
                      <div className="text-sm text-muted-foreground">
                        {check.message}
                      </div>
                    </div>
                  </div>
                  <Badge 
                    variant={check.status === 'healthy' ? 'default' : 'destructive'}
                    className="ml-2"
                  >
                    {check.status}
                  </Badge>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center text-muted-foreground">
              Failed to load dependency status
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}