<script setup lang="ts">
import { ImportExportRound } from "@vicons/material";

import StatusBtn from "@/components/status_btn/StatusBtn.vue";
import { useNATConfigStore } from "@/stores/status_nats";
import { IfaceZoneType } from "@landscape-router/types/api/schemas";

const natConfigStore = useNATConfigStore();

const iface_info = defineProps<{
  iface_name: string;
  zone: IfaceZoneType;
}>();

const status = natConfigStore.GET_STATUS_BY_IFACE_NAME(iface_info.iface_name);

const emit = defineEmits(["click"]);
</script>

<template>
  <StatusBtn
    v-if="iface_info.zone === IfaceZoneType.wan"
    :status="status"
    @click="emit('click')"
  >
    <template #btn-icon>
      <n-icon>
        <ImportExportRound />
      </n-icon>
    </template>
  </StatusBtn>
</template>
