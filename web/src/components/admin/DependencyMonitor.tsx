import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { apiClient } from '@/lib/api/client'
import { 
  Database, 
  Server, 
  CheckCircle2, 
  XCircle, 
  AlertTriangle, 
  Clock,
  Activity,
  Shield,
  Settings,
  Network
} from 'lucide-react'

import type { components } from '@/types/api'

type DetailedHealth = components['schemas']['DetailedHealthResponse']
type ComponentHealth = components['schemas']['ComponentHealth']

// Type for probe responses that return unknown data
interface ProbeResponse {
  probe?: string
  status?: string
  timestamp?: string
  checks?: Record<string, ComponentHealth>
}

export function DependencyMonitor() {
  const detailedHealthQuery = useQuery({
    queryKey: ['health', 'detailed'],
    queryFn: () => apiClient.getDetailedHealth(),
    refetchInterval: 15000,
  })

  const readinessQuery = useQuery({
    queryKey: ['health', 'readiness'],
    queryFn: () => apiClient.getReadinessProbe(),
    refetchInterval: 15000,
  })

  const startupQuery = useQuery({
    queryKey: ['health', 'startup'],
    queryFn: () => apiClient.getStartupProbe(),
    refetchInterval: 20000,
  })

  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy':
        return <CheckCircle2 className="h-5 w-5 text-green-600" />
      case 'unhealthy':
        return <XCircle className="h-5 w-5 text-red-600" />
      case 'degraded':
        return <AlertTriangle className="h-5 w-5 text-yellow-600" />
      default:
        return <Activity className="h-5 w-5 text-gray-600" />
    }
  }

  const getStatusBadge = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy':
        return <Badge className="bg-green-100 text-green-800">{status}</Badge>
      case 'unhealthy':
        return <Badge variant="destructive">{status}</Badge>
      case 'degraded':
        return <Badge className="bg-yellow-100 text-yellow-800">{status}</Badge>
      default:
        return <Badge variant="secondary">{status}</Badge>
    }
  }

  const getDependencyIcon = (name: string) => {
    switch (name.toLowerCase()) {
      case 'database':
        return <Database className="h-5 w-5 text-blue-600" />
      case 'application':
        return <Server className="h-5 w-5 text-purple-600" />
      case 'schema':
        return <Settings className="h-5 w-5 text-green-600" />
      default:
        return <Network className="h-5 w-5 text-gray-600" />
    }
  }

  const calculateOverallHealth = () => {
    const allChecks: ComponentHealth[] = [
      ...(detailedHealthQuery.data?.data?.checks ? Object.values(detailedHealthQuery.data.data.checks) : []),
      ...(readinessQuery.data?.data?.checks ? Object.values(readinessQuery.data.data.checks) : []),
      ...(startupQuery.data?.data?.checks ? Object.values(startupQuery.data.data.checks) : [])
    ]

    if (allChecks.length === 0) return { percentage: 0, status: 'unknown' }

    const healthyCount = allChecks.filter(check => check.status === 'healthy').length
    const percentage = Math.round((healthyCount / allChecks.length) * 100)
    
    let status = 'healthy'
    if (percentage < 50) status = 'unhealthy'
    else if (percentage < 90) status = 'degraded'

    return { percentage, status }
  }

  const overallHealth = calculateOverallHealth()

  return (
    <div className="space-y-6">
      {/* Overall Dependency Health */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Shield className="h-5 w-5" />
            <span>Dependency Health Overview</span>
          </CardTitle>
          <CardDescription>
            Real-time monitoring of critical system dependencies
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center space-x-3">
              {getStatusIcon(overallHealth.status)}
              <div>
                <div className="text-2xl font-bold">{overallHealth.percentage}%</div>
                <div className="text-sm text-muted-foreground">Dependencies Healthy</div>
              </div>
            </div>
            {getStatusBadge(overallHealth.status)}
          </div>
          <Progress value={overallHealth.percentage} className="h-3" />
          <div className="mt-2 text-xs text-muted-foreground">
            Last updated: {new Date().toLocaleTimeString()}
          </div>
        </CardContent>
      </Card>

      {/* Detailed Dependency Status */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Core Dependencies */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Database className="h-5 w-5" />
              <span>Core Dependencies</span>
            </CardTitle>
            <CardDescription>
              Essential services required for application operation
            </CardDescription>
          </CardHeader>
          <CardContent>
            {detailedHealthQuery.isLoading ? (
              <div className="flex items-center space-x-2">
                <Activity className="h-4 w-4 animate-spin" />
                <span>Loading dependency status...</span>
              </div>
            ) : detailedHealthQuery.error ? (
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  Failed to load core dependencies: {detailedHealthQuery.error.message}
                </AlertDescription>
              </Alert>
            ) : (
              <div className="space-y-4">
                {detailedHealthQuery.data?.data?.checks && Object.entries(detailedHealthQuery.data.data.checks).map(([name, check]: [string, ComponentHealth]) => (
                  <div key={name} className="flex items-start justify-between p-4 border rounded-lg">
                    <div className="flex items-start space-x-3">
                      {getDependencyIcon(name)}
                      <div className="space-y-1">
                        <div className="flex items-center space-x-2">
                          <h4 className="font-semibold capitalize">{name}</h4>
                          {getStatusBadge(check.status)}
                        </div>
                        <p className="text-sm text-muted-foreground">{check.message}</p>
                        {check.details && (
                          <div className="text-xs text-muted-foreground bg-gray-50 p-2 rounded mt-2">
                            {Object.entries(check.details as Record<string, any>).map(([key, value]) => (
                              <div key={key} className="flex justify-between">
                                <span className="font-medium">{key.replace('_', ' ')}:</span>
                                <span>{String(value)}</span>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                    {getStatusIcon(check.status)}
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Application Dependencies */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Server className="h-5 w-5" />
              <span>Application Dependencies</span>
            </CardTitle>
            <CardDescription>
              Application-level dependencies and configuration status
            </CardDescription>
          </CardHeader>
          <CardContent>
            {readinessQuery.isLoading ? (
              <div className="flex items-center space-x-2">
                <Activity className="h-4 w-4 animate-spin" />
                <span>Loading application dependencies...</span>
              </div>
            ) : readinessQuery.error ? (
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  Failed to load application dependencies: {readinessQuery.error.message}
                </AlertDescription>
              </Alert>
            ) : (
              <div className="space-y-4">
                {(() => {
                  const readinessData = readinessQuery.data?.data as ProbeResponse
                  return readinessData?.checks && Object.entries(readinessData.checks).map(([name, check]: [string, ComponentHealth]) => (
                  <div key={`readiness-${name}`} className="flex items-start justify-between p-4 border rounded-lg">
                    <div className="flex items-start space-x-3">
                      {getDependencyIcon(name)}
                      <div className="space-y-1">
                        <div className="flex items-center space-x-2">
                          <h4 className="font-semibold capitalize">{name}</h4>
                          {getStatusBadge(check.status)}
                        </div>
                        <p className="text-sm text-muted-foreground">{check.message}</p>
                        {check.details && (
                          <div className="text-xs text-muted-foreground bg-gray-50 p-2 rounded mt-2">
                            {Object.entries(check.details as Record<string, any>).map(([key, value]) => (
                              <div key={key} className="flex justify-between">
                                <span className="font-medium">{key.replace('_', ' ')}:</span>
                                <span>{String(value)}</span>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                    {getStatusIcon(check.status)}
                  </div>
                  ))
                })()}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Startup Dependencies */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Settings className="h-5 w-5" />
            <span>Startup Dependencies</span>
          </CardTitle>
          <CardDescription>
            Initialization and schema validation dependencies
          </CardDescription>
        </CardHeader>
        <CardContent>
          {startupQuery.isLoading ? (
            <div className="flex items-center space-x-2">
              <Activity className="h-4 w-4 animate-spin" />
              <span>Loading startup dependencies...</span>
            </div>
          ) : startupQuery.error ? (
            <Alert>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                Failed to load startup dependencies: {startupQuery.error.message}
              </AlertDescription>
            </Alert>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {(() => {
                const startupData = startupQuery.data?.data as ProbeResponse
                return startupData?.checks && Object.entries(startupData.checks).map(([name, check]: [string, ComponentHealth]) => (
                <div key={`startup-${name}`} className="flex items-start justify-between p-4 border rounded-lg">
                  <div className="flex items-start space-x-3">
                    {getDependencyIcon(name)}
                    <div className="space-y-1">
                      <div className="flex items-center space-x-2">
                        <h4 className="font-semibold capitalize">{name}</h4>
                        {getStatusBadge(check.status)}
                      </div>
                      <p className="text-sm text-muted-foreground">{check.message}</p>
                      {check.details && (
                        <div className="text-xs text-muted-foreground bg-gray-50 p-2 rounded mt-2">
                          {Object.entries(check.details as Record<string, any>).map(([key, value]) => (
                            <div key={key} className="flex justify-between">
                              <span className="font-medium">{key.replace('_', ' ')}:</span>
                              <span>{String(value)}</span>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                  {getStatusIcon(check.status)}
                </div>
                ))
              })()}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Dependency Timeline */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center space-x-2">
            <Clock className="h-5 w-5" />
            <span>Last Updated</span>
          </CardTitle>
          <CardDescription>
            Timestamp information for dependency health checks
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
            <div className="flex items-center justify-between p-3 bg-gray-50 rounded">
              <span className="font-medium">Detailed Health:</span>
              <span className="text-muted-foreground">
                {detailedHealthQuery.data?.data.timestamp ? 
                  new Date(detailedHealthQuery.data.data.timestamp).toLocaleTimeString() : 
                  'Not available'}
              </span>
            </div>
            <div className="flex items-center justify-between p-3 bg-gray-50 rounded">
              <span className="font-medium">Readiness Probe:</span>
              <span className="text-muted-foreground">
                {(() => {
                  const readinessData = readinessQuery.data?.data as ProbeResponse
                  return readinessData?.timestamp ? 
                    new Date(readinessData.timestamp).toLocaleTimeString() : 
                    'Not available'
                })()}
              </span>
            </div>
            <div className="flex items-center justify-between p-3 bg-gray-50 rounded">
              <span className="font-medium">Startup Probe:</span>
              <span className="text-muted-foreground">
                {(() => {
                  const startupData = startupQuery.data?.data as ProbeResponse
                  return startupData?.timestamp ? 
                    new Date(startupData.timestamp).toLocaleTimeString() : 
                    'Not available'
                })()}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}