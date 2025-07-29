import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { apiClient } from '@/lib/api/client'
import { toast } from 'sonner'
import { 
  Bell, 
  CheckCircle, 
  XCircle, 
  AlertTriangle,
  Activity,
  Database,
  Server,
  Zap,
  Users,
  Settings,
  Trash2,
  Eye,
  EyeOff
} from 'lucide-react'
import { useEffect, useState, useMemo } from 'react'
import { formatDistanceToNow } from 'date-fns'

import type { components } from '@/types/api'

type TaskStats = components['schemas']['TaskStats']

interface Notification {
  id: string
  type: 'task' | 'health' | 'user' | 'system'
  severity: 'info' | 'warning' | 'error' | 'success'
  title: string
  message: string
  timestamp: Date
  read: boolean
  actionable?: boolean
}

export function RealTimeNotifications() {
  const [notifications, setNotifications] = useState<Notification[]>([])
  const [showRead, setShowRead] = useState(false)

  const taskStatsQuery = useQuery({
    queryKey: ['tasks', 'stats'],
    queryFn: () => apiClient.getTaskStats(),
    refetchInterval: 5000, // Very frequent for notifications
  })

  const healthQuery = useQuery({
    queryKey: ['health', 'basic'],
    queryFn: () => apiClient.getHealth(),
    refetchInterval: 10000,
  })

  // const detailedHealthQuery = useQuery({
  //   queryKey: ['health', 'detailed'],
  //   queryFn: () => apiClient.getDetailedHealth(),
  //   refetchInterval: 15000,
  // })

  // Previous values to detect changes
  const [prevTaskStats, setPrevTaskStats] = useState<TaskStats | null>(null)
  const [prevHealthStatus, setPrevHealthStatus] = useState<string | null>(null)

  // Generate notifications based on data changes
  useEffect(() => {
    const currentStats = taskStatsQuery.data?.data
    const currentHealth = healthQuery.data?.data?.status

    // Task notifications
    if (prevTaskStats && currentStats) {
      // New failed tasks
      if (currentStats.failed > prevTaskStats.failed) {
        const newFailures = currentStats.failed - prevTaskStats.failed
        addNotification({
          type: 'task',
          severity: 'error',
          title: 'Task Failures Detected',
          message: `${newFailures} task(s) have failed and may need attention`,
          actionable: true
        })
      }

      // Tasks completed
      if (currentStats.completed > prevTaskStats.completed) {
        const newCompletions = currentStats.completed - prevTaskStats.completed
        if (newCompletions >= 5) { // Only notify for significant completions
          addNotification({
            type: 'task',
            severity: 'success',
            title: 'Tasks Completed',
            message: `${newCompletions} task(s) completed successfully`,
            actionable: false
          })
        }
      }

      // High pending task count
      if (currentStats.pending > 50 && prevTaskStats.pending <= 50) {
        addNotification({
          type: 'task',
          severity: 'warning',
          title: 'High Task Queue',
          message: `${currentStats.pending} tasks are pending. Consider scaling workers.`,
          actionable: true
        })
      }
    }

    // Health notifications
    if (prevHealthStatus && currentHealth && prevHealthStatus !== currentHealth) {
      if (currentHealth === 'unhealthy' && prevHealthStatus === 'healthy') {
        addNotification({
          type: 'health',
          severity: 'error',
          title: 'System Health Critical',
          message: 'Application health status changed to unhealthy',
          actionable: true
        })
      } else if (currentHealth === 'healthy' && prevHealthStatus === 'unhealthy') {
        addNotification({
          type: 'health',
          severity: 'success',
          title: 'System Health Restored',
          message: 'Application health status restored to healthy',
          actionable: false
        })
      }
    }

    // Update previous values
    if (currentStats) setPrevTaskStats(currentStats)
    if (currentHealth) setPrevHealthStatus(currentHealth)
  }, [taskStatsQuery.data, healthQuery.data, prevTaskStats, prevHealthStatus])

  // Generate periodic system notifications
  useEffect(() => {
    const interval = setInterval(() => {
      // Generate random system notifications for demonstration
      const notificationTypes = [
        {
          type: 'system' as const,
          severity: 'info' as const,
          title: 'System Checkpoint',
          message: 'Regular system checkpoint completed successfully'
        },
        {
          type: 'user' as const,
          severity: 'info' as const,
          title: 'User Activity',
          message: 'Peak user activity detected during business hours'
        }
      ]

      // Add random notification occasionally
      if (Math.random() > 0.7) {
        const randomNotification = notificationTypes[Math.floor(Math.random() * notificationTypes.length)]
        addNotification(randomNotification)
      }
    }, 30000) // Every 30 seconds

    return () => clearInterval(interval)
  }, [])

  const addNotification = (notification: Omit<Notification, 'id' | 'timestamp' | 'read'>) => {
    const newNotification: Notification = {
      ...notification,
      id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
      timestamp: new Date(),
      read: false
    }

    setNotifications(prev => [newNotification, ...prev.slice(0, 49)]) // Keep last 50

    // Show toast notification
    const toastProps = {
      description: notification.message,
      duration: 5000,
    }

    switch (notification.severity) {
      case 'error':
        toast.error(notification.title, toastProps)
        break
      case 'warning':
        toast.warning(notification.title, toastProps)
        break
      case 'success':
        toast.success(notification.title, toastProps)
        break
      default:
        toast.info(notification.title, toastProps)
    }
  }

  const markAsRead = (id: string) => {
    setNotifications(prev => 
      prev.map(notification => 
        notification.id === id ? { ...notification, read: true } : notification
      )
    )
  }

  const markAllAsRead = () => {
    setNotifications(prev => 
      prev.map(notification => ({ ...notification, read: true }))
    )
  }

  const clearAll = () => {
    setNotifications([])
  }

  const getIcon = (type: string, severity: string) => {
    if (severity === 'error') return <XCircle className="h-4 w-4 text-red-600" />
    if (severity === 'warning') return <AlertTriangle className="h-4 w-4 text-yellow-600" />
    if (severity === 'success') return <CheckCircle className="h-4 w-4 text-green-600" />

    switch (type) {
      case 'task': return <Activity className="h-4 w-4 text-blue-600" />
      case 'health': return <Database className="h-4 w-4 text-purple-600" />
      case 'user': return <Users className="h-4 w-4 text-indigo-600" />
      case 'system': return <Server className="h-4 w-4 text-gray-600" />
      default: return <Bell className="h-4 w-4 text-gray-600" />
    }
  }

  const getBadgeVariant = (severity: string) => {
    switch (severity) {
      case 'error': return 'destructive'
      case 'warning': return 'secondary'
      case 'success': return 'default'
      default: return 'outline'
    }
  }

  const filteredNotifications = useMemo(() => {
    return showRead ? notifications : notifications.filter(n => !n.read)
  }, [notifications, showRead])

  const unreadCount = notifications.filter(n => !n.read).length

  return (
    <div className="space-y-6">
      {/* Notification Controls */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center space-x-2">
                <Bell className="h-5 w-5" />
                <span>Real-time Notifications</span>
                {unreadCount > 0 && (
                  <Badge className="bg-red-500 text-white">{unreadCount}</Badge>
                )}
              </CardTitle>
              <CardDescription>
                Live system alerts and activity notifications
              </CardDescription>
            </div>
            <div className="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowRead(!showRead)}
                className="flex items-center space-x-1"
              >
                {showRead ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                <span>{showRead ? 'Hide Read' : 'Show All'}</span>
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={markAllAsRead}
                disabled={unreadCount === 0}
              >
                Mark All Read
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={clearAll}
                disabled={notifications.length === 0}
                className="text-red-600 hover:text-red-700"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-4">
            <div className="text-center p-3 bg-red-50 rounded-lg">
              <div className="text-lg font-semibold text-red-600">
                {notifications.filter(n => n.severity === 'error').length}
              </div>
              <div className="text-sm text-muted-foreground">Errors</div>
            </div>
            <div className="text-center p-3 bg-yellow-50 rounded-lg">
              <div className="text-lg font-semibold text-yellow-600">
                {notifications.filter(n => n.severity === 'warning').length}
              </div>
              <div className="text-sm text-muted-foreground">Warnings</div>
            </div>
            <div className="text-center p-3 bg-green-50 rounded-lg">
              <div className="text-lg font-semibold text-green-600">
                {notifications.filter(n => n.severity === 'success').length}
              </div>
              <div className="text-sm text-muted-foreground">Success</div>
            </div>
            <div className="text-center p-3 bg-blue-50 rounded-lg">
              <div className="text-lg font-semibold text-blue-600">
                {notifications.filter(n => n.severity === 'info').length}
              </div>
              <div className="text-sm text-muted-foreground">Info</div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Notifications List */}
      <Card>
        <CardHeader>
          <CardTitle>
            Recent Notifications 
            <span className="text-sm font-normal text-muted-foreground ml-2">
              ({filteredNotifications.length} {showRead ? 'total' : 'unread'})
            </span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {filteredNotifications.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Bell className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <div className="text-lg font-medium">No notifications</div>
              <div className="text-sm">
                {showRead ? 'No notifications to display' : 'All caught up! No unread notifications.'}
              </div>
            </div>
          ) : (
            <ScrollArea className="h-96">
              <div className="space-y-4">
                {filteredNotifications.map((notification, index) => (
                  <div key={notification.id}>
                    <div 
                      className={`flex items-start space-x-3 p-3 rounded-lg cursor-pointer transition-colors ${
                        notification.read 
                          ? 'bg-gray-50 opacity-75' 
                          : 'bg-blue-50 hover:bg-blue-100'
                      }`}
                      onClick={() => !notification.read && markAsRead(notification.id)}
                    >
                      <div className="flex-shrink-0 mt-1">
                        {getIcon(notification.type, notification.severity)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between mb-1">
                          <div className="flex items-center space-x-2">
                            <h4 className={`text-sm font-medium ${notification.read ? 'text-gray-600' : 'text-gray-900'}`}>
                              {notification.title}
                            </h4>
                            <Badge variant={getBadgeVariant(notification.severity)} className="text-xs">
                              {notification.severity}
                            </Badge>
                            {notification.actionable && (
                              <Badge variant="outline" className="text-xs">
                                Action needed
                              </Badge>
                            )}
                          </div>
                          {!notification.read && (
                            <div className="w-2 h-2 bg-blue-600 rounded-full flex-shrink-0"></div>
                          )}
                        </div>
                        <p className={`text-sm ${notification.read ? 'text-gray-500' : 'text-gray-700'}`}>
                          {notification.message}
                        </p>
                        <div className="flex items-center justify-between mt-2">
                          <span className="text-xs text-muted-foreground">
                            {formatDistanceToNow(notification.timestamp, { addSuffix: true })}
                          </span>
                          <Badge variant="outline" className="text-xs capitalize">
                            {notification.type}
                          </Badge>
                        </div>
                      </div>
                    </div>
                    {index < filteredNotifications.length - 1 && <Separator className="my-2" />}
                  </div>
                ))}
              </div>
            </ScrollArea>
          )}
        </CardContent>
      </Card>

      {/* Notification Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Settings className="h-5 w-5" />
            <span>Notification Settings</span>
          </CardTitle>
          <CardDescription>
            Configure how you receive real-time notifications
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <h4 className="font-medium">Notification Types</h4>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm">Task failures</span>
                  <Badge className="bg-red-100 text-red-800">Enabled</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm">Health alerts</span>
                  <Badge className="bg-red-100 text-red-800">Enabled</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm">Task completions</span>
                  <Badge className="bg-green-100 text-green-800">Enabled</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm">System updates</span>
                  <Badge variant="outline">Disabled</Badge>
                </div>
              </div>
            </div>
            
            <div className="space-y-4">
              <h4 className="font-medium">Update Frequency</h4>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm">Task statistics</span>
                  <Badge variant="outline">5 seconds</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm">Health checks</span>
                  <Badge variant="outline">10 seconds</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm">System status</span>
                  <Badge variant="outline">30 seconds</Badge>
                </div>
              </div>
            </div>
          </div>

          <div className="mt-6 pt-4 border-t text-xs text-muted-foreground">
            <div className="flex items-center space-x-2">
              <Zap className="h-4 w-4" />
              <span>Real-time notifications are active. Data updates automatically based on the configured intervals.</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}