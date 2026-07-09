<script setup lang="ts">
import { IfaceZoneType } from "@landscape-router/types/api/schemas";
import { Cut } from "@vicons/tabler";

import StatusBtn from "@/components/status_btn/StatusBtn.vue";
import { useMSSClampConfigStore } from "@/stores/status_mss_clamp";

const mssclampConfigStore = useMSSClampConfigStore();

const iface_info = defineProps<{
  iface_name: string;
  zone: IfaceZoneType;
}>();

const status = mssclampConfigStore.GET_STATUS_BY_IFACE_NAME(
  iface_info.iface_name,
);

const emit = defineEmits(["click"]);
</script>

<template>
  <StatusBtn :status="status" @click="emit('click')">
    <template #btn-icon>
      <n-icon>
        <Cut />
      </n-icon>
    </template>
  </StatusBtn>
</template>
