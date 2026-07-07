import { createRouter, createWebHashHistory } from 'vue-router'
import Dashboard from '../views/Dashboard.vue'
import ReviewHistory from '../views/ReviewHistory.vue'
import Configuration from '../views/Configuration.vue'
import QueueMonitor from '../views/QueueMonitor.vue'
import LlmStatus from '../views/LlmStatus.vue'
import SystemLogs from '../views/SystemLogs.vue'
import ExpertsManagement from '../views/ExpertsManagement.vue'

const routes = [
  { path: '/', redirect: '/dashboard' },
  { path: '/dashboard', name: 'Dashboard', component: Dashboard },
  { path: '/history', name: 'ReviewHistory', component: ReviewHistory },
  { path: '/config', name: 'Configuration', component: Configuration },
  { path: '/queue', name: 'QueueMonitor', component: QueueMonitor },
  { path: '/llm', name: 'LlmStatus', component: LlmStatus },
  { path: '/logs', name: 'SystemLogs', component: SystemLogs },
  { path: '/experts', name: 'ExpertsManagement', component: ExpertsManagement },
]

export default createRouter({
  history: createWebHashHistory(),
  routes,
})
