<script setup lang="ts">
import { computed } from 'vue'
import { Loading } from '@element-plus/icons-vue'

type BadgeStatus = 'success' | 'warning' | 'error' | 'info' | 'offline' | 'running' | 'queued' | 'failed' | 'completed' | 'cancelled' | 'skipped'

interface Props {
  status: BadgeStatus
  size?: 'small' | 'default'
  dotOnly?: boolean
  showText?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  size: 'small',
  dotOnly: false,
  showText: true,
})

const statusConfigMap: Record<string, { type: string; text: string; color: string; effect?: string }> = {
  success:    { type: 'success', text: 'Operational',  color: 'var(--success)' },
  running:    { type: 'success', text: 'In Progress',  color: 'var(--success)' },
  completed:  { type: 'success', text: 'Completed',    color: 'var(--success)' },
  warning:    { type: 'warning', text: 'Degraded',     color: 'var(--warning)' },
  queued:     { type: 'info',    text: 'Queued',       color: 'var(--info)' },
  info:       { type: 'info',    text: 'Info',         color: 'var(--info)' },
  error:      { type: 'danger',  text: 'Error',        color: 'var(--error)' },
  failed:     { type: 'danger',  text: 'Failed',       color: 'var(--error)' },
  offline:    { type: 'info',    text: 'Offline',      color: 'var(--offline)' },
  cancelled:  { type: 'info',    text: 'Cancelled',    color: 'var(--text-secondary)', effect: 'plain' },
  skipped:    { type: 'info',    text: 'Skipped',      color: 'var(--text-secondary)', effect: 'plain' },
}

const config = computed(() => statusConfigMap[props.status] || { type: 'info', text: props.status, color: 'var(--text-secondary)' })

const dotColor = computed(() => config.value.color)
const isRunning = computed(() => props.status === 'running')
</script>

<template>
  <!-- dotOnly variant -->
  <span
    v-if="dotOnly"
    class="status-badge-dot-only"
    :class="{ 'is-running': isRunning }"
    :style="{ backgroundColor: dotColor }"
    aria-hidden="true"
  />

  <!-- ElTag variant (ReviewHistory style) -->
  <el-tag
    v-else-if="!showText && !dotOnly"
    :type="config.type as any"
    :effect="config.effect || 'light'"
    :size="size"
    class="status-badge-tag"
  >
    <el-icon v-if="status === 'running'" class="is-loading"><Loading /></el-icon>
    {{ config.text }}
  </el-tag>

  <!-- Custom badge variant (Dashboard style with text) -->
  <span
    v-else
    class="status-badge"
    :class="[`size-${size}`, `status-${status}`]"
  >
    <span
      class="status-badge-dot"
      :class="{ 'is-running': isRunning }"
      :style="{ backgroundColor: dotColor }"
      aria-hidden="true"
    />
    <span v-if="showText" class="status-badge-text">{{ config.text }}</span>
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.status-badge-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-badge-dot.is-running {
  animation: pulse 2s infinite;
}

.status-badge-text {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.status-badge.size-small .status-badge-dot {
  width: 6px;
  height: 6px;
}
.status-badge.size-small .status-badge-text {
  font-size: 11px;
}

.status-badge.size-default .status-badge-dot {
  width: 8px;
  height: 8px;
}
.status-badge.size-default .status-badge-text {
  font-size: 13px;
}

.status-badge-dot-only {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.status-badge-dot-only.is-running {
  animation: pulse 2s infinite;
}

.status-badge-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.is-loading {
  animation: rotating 2s linear infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

@keyframes rotating {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
