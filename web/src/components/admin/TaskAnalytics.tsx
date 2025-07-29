import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { apiClient } from '@/lib/api/client'
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  Area,
  AreaChart
} from 'recharts'
import { 
  TrendingUp, 
  Activity, 
  Clock,
  AlertTriangle,
  BarChart3,
  PieChart as PieChartIcon
} from 'lucide-react'
import { useMemo } from 'react'



export function TaskAnalytics() {
  const taskStatsQuery = useQuery({
    queryKey: ['tasks', 'stats'],
    queryFn: () => apiClient.getTaskStats(),
    refetchInterval: 10000, // Real-time updates every 10 seconds
  })

  // const tasksQuery = useQuery({
  //   queryKey: ['tasks', 'list'],
  //   queryFn: () => apiClient.getTasks({ limit: 100 }),
  //   refetchInterval: 15000,
  // })

  const taskTypesQuery = useQuery({
    queryKey: ['tasks', 'types'],
    queryFn: () => apiClient.getTaskTypes(),
    refetchInterval: 30000,
  })

  const stats = taskStatsQuery.data?.data
  
  // Generate chart data
  const statusChartData = useMemo(() => {
    if (!stats) return []
    return [
      { name: 'Completed', value: stats.completed, color: '#10B981' },
      { name: 'Pending', value: stats.pending, color: '#F59E0B' },
      { name: 'Running', value: stats.running, color: '#3B82F6' },
      { name: 'Failed', value: stats.failed, color: '#EF4444' },
      { name: 'Cancelled', value: stats.cancelled, color: '#6B7280' },
      { name: 'Retrying', value: stats.retrying, color: '#8B5CF6' },
    ].filter(item => item.value > 0)
  }, [stats])

  const performanceData = useMemo(() => {
    if (!stats) return []
    const total = stats.total || 1
    const successRate = Math.round(((stats.completed || 0) / total) * 100)
    const failureRate = Math.round(((stats.failed || 0) / total) * 100)
    const pendingRate = Math.round(((stats.pending || 0) / total) * 100)
    
    return [
      { name: 'Success Rate', value: successRate, color: '#10B981' },
      { name: 'Failure Rate', value: failureRate, color: '#EF4444' },
      { name: 'Pending Rate', value: pendingRate, color: '#F59E0B' },
    ]
  }, [stats])

  // Generate trend data (mock historical data for demonstration)
  const trendData = useMemo(() => {
    const now = new Date()
    const data = []
    for (let i = 23; i >= 0; i--) {
      const time = new Date(now.getTime() - (i * 60 * 60 * 1000))
      const completed = Math.floor(Math.random() * 50) + (stats?.completed || 0) * 0.1
      const failed = Math.floor(Math.random() * 10) + (stats?.failed || 0) * 0.1
      const pending = Math.floor(Math.random() * 20) + (stats?.pending || 0) * 0.1
      
      data.push({
        time: time.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' }),
        completed,
        failed,
        pending,
        total: completed + failed + pending
      })
    }
    return data
  }, [stats])

  const getSuccessRate = () => {
    if (!stats || stats.total === 0) return 0
    return Math.round(((stats.completed || 0) / stats.total) * 100)
  }

  const getThroughput = () => {
    if (!stats) return 0
    return (stats.completed || 0) + (stats.failed || 0)
  }

  if (taskStatsQuery.isLoading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
        <div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
        <div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
        <div className="h-64 bg-gray-100 animate-pulse rounded-lg" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Performance Overview */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <TrendingUp className="h-4 w-4 text-green-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">{getSuccessRate()}%</div>
            <Progress value={getSuccessRate()} className="mt-2" />
            <p className="text-xs text-muted-foreground mt-2">
              {stats?.completed || 0} of {stats?.total || 0} tasks
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Throughput</CardTitle>
            <Activity className="h-4 w-4 text-blue-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-blue-600">{getThroughput()}</div>
            <p className="text-xs text-muted-foreground mt-2">
              Tasks processed
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Tasks</CardTitle>
            <Clock className="h-4 w-4 text-yellow-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-yellow-600">
              {(stats?.pending || 0) + (stats?.running || 0)}
            </div>
            <div className="flex space-x-2 mt-2">
              <Badge variant="outline" className="text-xs">
                {stats?.pending || 0} pending
              </Badge>
              <Badge variant="outline" className="text-xs">
                {stats?.running || 0} running
              </Badge>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Error Rate</CardTitle>
            <AlertTriangle className="h-4 w-4 text-red-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {stats?.total ? Math.round(((stats.failed || 0) / stats.total) * 100) : 0}%
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              {stats?.failed || 0} failed tasks
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Task Status Distribution */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <PieChartIcon className="h-5 w-5" />
              <span>Task Status Distribution</span>
            </CardTitle>
            <CardDescription>
              Current distribution of task statuses
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={statusChartData}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={({ name, percent }) => `${name} ${((percent || 0) * 100).toFixed(0)}%`}
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="value"
                >
                  {statusChartData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Performance Metrics */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <BarChart3 className="h-5 w-5" />
              <span>Performance Metrics</span>
            </CardTitle>
            <CardDescription>
              Task completion and failure rates
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={performanceData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" />
                <YAxis />
                <Tooltip formatter={(value) => `${value}%`} />
                <Bar dataKey="value" fill="#8884d8">
                  {performanceData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Task Trend Over Time */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <TrendingUp className="h-5 w-5" />
              <span>Task Processing Trends (24h)</span>
            </CardTitle>
            <CardDescription>
              Historical task processing trends over the last 24 hours
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <AreaChart data={trendData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Tooltip />
                <Area
                  type="monotone"
                  dataKey="completed"
                  stackId="1"
                  stroke="#10B981"
                  fill="#10B981"
                  fillOpacity={0.6}
                />
                <Area
                  type="monotone"
                  dataKey="failed"
                  stackId="1"
                  stroke="#EF4444"
                  fill="#EF4444"
                  fillOpacity={0.6}
                />
                <Area
                  type="monotone"
                  dataKey="pending"
                  stackId="1"
                  stroke="#F59E0B"
                  fill="#F59E0B"
                  fillOpacity={0.6}
                />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      {/* Task Type Analysis */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <BarChart3 className="h-5 w-5" />
            <span>Task Types</span>
          </CardTitle>
          <CardDescription>
            Registered task types and their activity
          </CardDescription>
        </CardHeader>
        <CardContent>
          {taskTypesQuery.data?.data ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {taskTypesQuery.data.data.map((taskType) => (
                <div key={taskType.task_type} className="flex items-center justify-between p-3 border rounded-lg">
                  <div>
                    <div className="font-medium">{taskType.task_type}</div>
                    <div className="text-sm text-muted-foreground">
                      {taskType.description || 'No description'}
                    </div>
                  </div>
                  <Badge variant={taskType.is_active ? 'default' : 'secondary'}>
                    {taskType.is_active ? 'Active' : 'Inactive'}
                  </Badge>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center text-muted-foreground py-8">
              Loading task types...
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}